use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::ocp_ocl::diag::Diagnostic;
use crate::ocp_ocl::hir::{build_hir_and_hash, canonical_hir_bytes, HIR_SCHEMA_VERSION};
use crate::ocp_ocl::parse::parse_program;

pub const COMPILE_CACHE_HASHER_VERSION_V1: &str = "sha256-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheKeyInput {
    pub compiler_version: String,
    pub ocl_version: String,
    pub lane_literal: String,
    pub lock_hash: String,
    pub trust_hash: String,
    pub deps_graph_hash: String,
    pub manifest_hash: String,
    pub schema_versions_of_packs: String,
    pub entry_module_hash: String,
    pub source_bundle_hash: String,
    pub cache_toggle_inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheArtifact {
    pub cache_key: String,
    pub ir_hash: String,
    pub canonical_hir_bytes: Vec<u8>,
    pub hit: bool,
    pub meta_path: PathBuf,
    pub hir_path: PathBuf,
}

#[derive(Debug)]
pub enum CompileCacheError {
    Diagnostic(Diagnostic),
    Io(io::Error),
}

impl std::fmt::Display for CompileCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Diagnostic(diag) => write!(f, "compile diagnostic: {}", diag.message),
            Self::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl std::error::Error for CompileCacheError {}

impl From<io::Error> for CompileCacheError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn compile_cache_key_sha256_v1(input: &CompileCacheKeyInput) -> String {
    let canonical = canonical_compile_cache_key_material(input);
    sha256_hex(canonical.as_bytes())
}

pub fn compile_with_cache(
    source: &str,
    file_id: u32,
    input: &CompileCacheKeyInput,
    cache_root: &Path,
) -> Result<CompileCacheArtifact, CompileCacheError> {
    let cache_key = compile_cache_key_sha256_v1(input);
    let entry_dir = cache_root.join("compile").join(&cache_key);
    let meta_path = entry_dir.join("meta.toml");
    let hir_path = entry_dir.join("hir.bin");

    if let Some(hit_artifact) = try_read_cache_hit(&cache_key, &meta_path, &hir_path) {
        return Ok(hit_artifact);
    }

    let program = parse_program(source, file_id).map_err(CompileCacheError::Diagnostic)?;
    let (hir, ir_hash) = build_hir_and_hash(&program).map_err(CompileCacheError::Diagnostic)?;
    let canonical_hir = canonical_hir_bytes(&hir);

    fs::create_dir_all(&entry_dir)?;
    fs::write(&hir_path, &canonical_hir)?;
    fs::write(
        &meta_path,
        encode_meta(&cache_key, &ir_hash, input, canonical_hir.len()),
    )?;

    Ok(CompileCacheArtifact {
        cache_key,
        ir_hash,
        canonical_hir_bytes: canonical_hir,
        hit: false,
        meta_path,
        hir_path,
    })
}

fn try_read_cache_hit(
    cache_key: &str,
    meta_path: &Path,
    hir_path: &Path,
) -> Option<CompileCacheArtifact> {
    let meta_raw = fs::read_to_string(meta_path).ok()?;
    let meta = parse_meta(&meta_raw)?;
    if meta.hasher_version != COMPILE_CACHE_HASHER_VERSION_V1 {
        return None;
    }
    if meta.hir_schema_version != HIR_SCHEMA_VERSION {
        return None;
    }
    if meta.cache_key != cache_key {
        return None;
    }
    let canonical_hir_bytes = fs::read(hir_path).ok()?;
    Some(CompileCacheArtifact {
        cache_key: cache_key.to_string(),
        ir_hash: meta.ir_hash,
        canonical_hir_bytes,
        hit: true,
        meta_path: meta_path.to_path_buf(),
        hir_path: hir_path.to_path_buf(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompileCacheMeta {
    hasher_version: String,
    hir_schema_version: u32,
    cache_key: String,
    ir_hash: String,
}

fn encode_meta(
    cache_key: &str,
    ir_hash: &str,
    input: &CompileCacheKeyInput,
    hir_bytes_len: usize,
) -> String {
    let mut out = String::new();
    out.push_str("hasher_version=");
    out.push_str(COMPILE_CACHE_HASHER_VERSION_V1);
    out.push('\n');
    out.push_str("hir_schema_version=");
    out.push_str(&HIR_SCHEMA_VERSION.to_string());
    out.push('\n');
    out.push_str("cache_key=");
    out.push_str(cache_key);
    out.push('\n');
    out.push_str("ir_hash=");
    out.push_str(ir_hash);
    out.push('\n');
    out.push_str("hir_bytes_len=");
    out.push_str(&hir_bytes_len.to_string());
    out.push('\n');
    out.push_str("compiler_version=");
    out.push_str(&input.compiler_version);
    out.push('\n');
    out.push_str("ocl_version=");
    out.push_str(&input.ocl_version);
    out.push('\n');
    out.push_str("lane_literal=");
    out.push_str(&input.lane_literal);
    out.push('\n');
    out
}

fn parse_meta(raw: &str) -> Option<CompileCacheMeta> {
    let mut hasher_version = None;
    let mut hir_schema_version = None;
    let mut cache_key = None;
    let mut ir_hash = None;
    for line in raw.lines() {
        let (name, value) = line.split_once('=')?;
        match name.trim() {
            "hasher_version" => hasher_version = Some(value.trim().to_string()),
            "hir_schema_version" => hir_schema_version = value.trim().parse::<u32>().ok(),
            "cache_key" => cache_key = Some(value.trim().to_string()),
            "ir_hash" => ir_hash = Some(value.trim().to_string()),
            _ => {}
        }
    }
    Some(CompileCacheMeta {
        hasher_version: hasher_version?,
        hir_schema_version: hir_schema_version?,
        cache_key: cache_key?,
        ir_hash: ir_hash?,
    })
}

fn canonical_compile_cache_key_material(input: &CompileCacheKeyInput) -> String {
    let mut toggles = input.cache_toggle_inputs.clone();
    toggles.sort();
    toggles.dedup();
    let toggle_join = toggles.join(",");
    let mut out = String::new();
    out.push_str("hasher_version=");
    out.push_str(COMPILE_CACHE_HASHER_VERSION_V1);
    out.push('\n');
    out.push_str("compiler_version=");
    out.push_str(&input.compiler_version);
    out.push('\n');
    out.push_str("ocl_version=");
    out.push_str(&input.ocl_version);
    out.push('\n');
    out.push_str("lane_literal=");
    out.push_str(&input.lane_literal);
    out.push('\n');
    out.push_str("lock_hash=");
    out.push_str(&input.lock_hash);
    out.push('\n');
    out.push_str("trust_hash=");
    out.push_str(&input.trust_hash);
    out.push('\n');
    out.push_str("deps_graph_hash=");
    out.push_str(&input.deps_graph_hash);
    out.push('\n');
    out.push_str("manifest_hash=");
    out.push_str(&input.manifest_hash);
    out.push('\n');
    out.push_str("schema_versions_of_packs=");
    out.push_str(&input.schema_versions_of_packs);
    out.push('\n');
    out.push_str("entry_module_hash=");
    out.push_str(&input.entry_module_hash);
    out.push('\n');
    out.push_str("source_bundle_hash=");
    out.push_str(&input.source_bundle_hash);
    out.push('\n');
    out.push_str("cache_toggle_inputs=");
    out.push_str(&toggle_join);
    out.push('\n');
    out
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    bytes_to_hex(digest.as_slice())
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
