use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::ocp::{
    execute_program, parse_program, typecheck_program, DiagPhase, Diagnostic, ErrorCode,
    ExecConfig, ExecOutput, Program, Span, Stmt,
};

#[derive(Debug, Clone)]
pub enum FixtureRunnerError {
    Io(String),
    Diag(Diagnostic),
}

impl From<Diagnostic> for FixtureRunnerError {
    fn from(value: Diagnostic) -> Self {
        Self::Diag(value)
    }
}

pub fn run_fixture_source(
    source: &str,
    file_id: u32,
    config: ExecConfig,
) -> Result<ExecOutput, FixtureRunnerError> {
    let program = parse_program(source, file_id)?;
    typecheck_program(&program)?;
    let out = execute_program(&program, config)?;
    Ok(out)
}

pub fn run_fixture_file(
    path: &Path,
    file_id: u32,
    config: ExecConfig,
) -> Result<ExecOutput, FixtureRunnerError> {
    let program = load_program_with_imports(path, file_id)?;
    typecheck_program(&program)?;
    let out = execute_program(&program, config)?;
    Ok(out)
}

fn load_program_with_imports(path: &Path, file_id: u32) -> Result<Program, FixtureRunnerError> {
    let root = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let root_canon = root.canonicalize().map_err(|e| {
        FixtureRunnerError::Io(format!(
            "failed to canonicalize fixture root '{}': {e}",
            root.display()
        ))
    })?;

    let entry_canon = path.canonicalize().map_err(|e| {
        FixtureRunnerError::Io(format!("failed to read fixture '{}': {e}", path.display()))
    })?;
    if !entry_canon.starts_with(&root_canon) {
        return Err(FixtureRunnerError::Diag(
            Diagnostic::new(
                ErrorCode::TImportNotFound,
                DiagPhase::Typecheck,
                Span::new(0, 0, 0, 0, 0),
                format!(
                    "entry '{}' is outside fixture root '{}'",
                    entry_canon.display(),
                    root_canon.display()
                ),
            )
            .with_hint("use entry file under fixture root"),
        ));
    }

    let mut loader = ModuleLoader {
        root_canon,
        next_file_id: file_id,
        visiting: HashSet::new(),
        loaded: HashSet::new(),
        statements: Vec::new(),
    };
    let entry_module_id = module_id_from_path(&loader.root_canon, &entry_canon)?;
    loader.load_module(entry_module_id, entry_canon)?;
    Ok(Program {
        statements: loader.statements,
    })
}

struct ModuleLoader {
    root_canon: PathBuf,
    next_file_id: u32,
    visiting: HashSet<String>,
    loaded: HashSet<String>,
    statements: Vec<Stmt>,
}

impl ModuleLoader {
    fn load_module(
        &mut self,
        module_id: String,
        file_path: PathBuf,
    ) -> Result<(), FixtureRunnerError> {
        if self.loaded.contains(&module_id) {
            return Ok(());
        }
        if self.visiting.contains(&module_id) {
            return Err(FixtureRunnerError::Diag(
                Diagnostic::new(
                    ErrorCode::TImportCycle,
                    DiagPhase::Typecheck,
                    Span::new(self.next_file_id, 0, 0, 0, 0),
                    format!("import cycle detected at module `{module_id}`"),
                )
                .with_hint(
                    "break circular imports by extracting shared code to one-way dependency",
                ),
            ));
        }
        self.visiting.insert(module_id.clone());

        let src = fs::read_to_string(&file_path).map_err(|e| {
            FixtureRunnerError::Io(format!(
                "failed to read module '{}': {e}",
                file_path.display()
            ))
        })?;
        let file_id = self.next_file_id;
        self.next_file_id = self.next_file_id.saturating_add(1);
        let program = parse_program(&src, file_id)?;

        for stmt in &program.statements {
            if let Stmt::ImportDecl { path, span } = stmt {
                if is_std_import(path) {
                    continue;
                }
                let child_module_id = module_id_from_import_path(path);
                let child_path = self.resolve_module_file(path, *span)?;
                self.load_module(child_module_id, child_path)?;
            }
        }

        for stmt in program.statements {
            if !matches!(stmt, Stmt::ImportDecl { .. }) {
                self.statements.push(stmt);
            }
        }

        self.visiting.remove(&module_id);
        self.loaded.insert(module_id);
        Ok(())
    }

    fn resolve_module_file(
        &self,
        import_path: &[String],
        span: Span,
    ) -> Result<PathBuf, FixtureRunnerError> {
        let mut candidate_ocp = self.root_canon.clone();
        for seg in import_path {
            candidate_ocp.push(seg);
        }
        candidate_ocp.set_extension("ocp");
        let mut candidate_ocp = candidate_ocp.clone();
        candidate_ocp.set_extension("ocp");
        let candidate = if candidate_ocp.exists() {
            candidate_ocp
        } else if candidate_ocp.exists() {
            candidate_ocp
        } else {
            return Err(FixtureRunnerError::Diag(
                Diagnostic::new(
                    ErrorCode::TImportNotFound,
                    DiagPhase::Typecheck,
                    span,
                    format!(
                        "import module `{}` not found at '{}' or '{}'",
                        module_id_from_import_path(import_path),
                        candidate_ocp.display(),
                        candidate_ocp.display()
                    ),
                )
                .with_hint("create missing module file or fix import path"),
            ));
        };

        let canonical = candidate.canonicalize().map_err(|e| {
            FixtureRunnerError::Io(format!(
                "failed to canonicalize module path '{}': {e}",
                candidate.display()
            ))
        })?;

        if !canonical.starts_with(&self.root_canon) {
            return Err(FixtureRunnerError::Diag(
                Diagnostic::new(
                    ErrorCode::TImportNotFound,
                    DiagPhase::Typecheck,
                    span,
                    format!(
                        "import path '{}' resolved outside fixture root",
                        canonical.display()
                    ),
                )
                .with_hint("use relative module path under fixture root only"),
            ));
        }

        Ok(canonical)
    }
}

fn module_id_from_import_path(path: &[String]) -> String {
    path.join(".")
}

fn is_std_import(path: &[String]) -> bool {
    matches!(path.first().map(String::as_str), Some("std"))
}

fn module_id_from_path(root: &Path, file_path: &Path) -> Result<String, FixtureRunnerError> {
    let rel = file_path.strip_prefix(root).map_err(|_| {
        FixtureRunnerError::Diag(
            Diagnostic::new(
                ErrorCode::TImportNotFound,
                DiagPhase::Typecheck,
                Span::new(0, 0, 0, 0, 0),
                format!(
                    "module path '{}' is outside root '{}'",
                    file_path.display(),
                    root.display()
                ),
            )
            .with_hint("use module files inside project root"),
        )
    })?;
    let mut segs = Vec::new();
    for comp in rel.components() {
        let raw = comp.as_os_str().to_string_lossy();
        segs.push(raw.to_string());
    }
    if let Some(last) = segs.last_mut() {
        if let Some(stripped) = last.strip_suffix(".ocp") {
            *last = stripped.to_string();
        } else if let Some(stripped) = last.strip_suffix(".ocp") {
            *last = stripped.to_string();
        }
    }
    Ok(segs.join("."))
}
