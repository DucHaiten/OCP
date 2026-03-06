use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeterminismError {
    UnsupportedPlatformProfile(String),
    UnsupportedRuntime {
        profile: String,
        os: String,
        arch: String,
    },
    PathEscape {
        raw: String,
    },
    InvalidUtf8,
    LocaleNotPinned(String),
    TimezoneNotPinned(String),
}

impl Display for DeterminismError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPlatformProfile(profile) => {
                write!(f, "unsupported platform profile `{profile}`")
            }
            Self::UnsupportedRuntime { profile, os, arch } => {
                write!(
                    f,
                    "runtime `{os}/{arch}` is outside supported deterministic profile `{profile}`"
                )
            }
            Self::PathEscape { raw } => write!(f, "path traversal escape denied for `{raw}`"),
            Self::InvalidUtf8 => write!(f, "input must be valid UTF-8"),
            Self::LocaleNotPinned(value) => {
                write!(f, "locale must be pinned to C.UTF-8 (found `{value}`)")
            }
            Self::TimezoneNotPinned(value) => {
                write!(f, "timezone must be pinned to UTC (found `{value}`)")
            }
        }
    }
}

impl std::error::Error for DeterminismError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedPlatformProfile {
    WinX64Ntfs,
    LinuxX64Ext4,
}

impl SupportedPlatformProfile {
    pub const fn id(self) -> &'static str {
        match self {
            Self::WinX64Ntfs => "win-x64-ntfs",
            Self::LinuxX64Ext4 => "linux-x64-ext4",
        }
    }

    pub const fn os(self) -> &'static str {
        match self {
            Self::WinX64Ntfs => "windows",
            Self::LinuxX64Ext4 => "linux",
        }
    }

    pub const fn arch(self) -> &'static str {
        "x86_64"
    }

    pub const fn case_sensitive(self) -> bool {
        match self {
            Self::WinX64Ntfs => false,
            Self::LinuxX64Ext4 => true,
        }
    }
}

pub fn supported_platform_profile_ids() -> Vec<&'static str> {
    vec![
        SupportedPlatformProfile::LinuxX64Ext4.id(),
        SupportedPlatformProfile::WinX64Ntfs.id(),
    ]
}

pub fn parse_supported_platform_profile(
    profile_id: &str,
) -> Result<SupportedPlatformProfile, DeterminismError> {
    match profile_id.trim() {
        "win-x64-ntfs" => Ok(SupportedPlatformProfile::WinX64Ntfs),
        "linux-x64-ext4" => Ok(SupportedPlatformProfile::LinuxX64Ext4),
        other => Err(DeterminismError::UnsupportedPlatformProfile(
            other.to_string(),
        )),
    }
}

pub fn profile_matches_runtime(profile: SupportedPlatformProfile, os: &str, arch: &str) -> bool {
    os == profile.os() && arch == profile.arch()
}

pub fn enforce_supported_runtime_profile(
    profile: SupportedPlatformProfile,
    os: &str,
    arch: &str,
) -> Result<(), DeterminismError> {
    if profile_matches_runtime(profile, os, arch) {
        return Ok(());
    }
    Err(DeterminismError::UnsupportedRuntime {
        profile: profile.id().to_string(),
        os: os.to_string(),
        arch: arch.to_string(),
    })
}

pub fn enforce_locale_timezone(locale: &str, timezone: &str) -> Result<(), DeterminismError> {
    if locale != "C.UTF-8" {
        return Err(DeterminismError::LocaleNotPinned(locale.to_string()));
    }
    if timezone != "UTC" {
        return Err(DeterminismError::TimezoneNotPinned(timezone.to_string()));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalTaskEvent {
    pub logical_time: u64,
    pub task_id: String,
    pub seq: u64,
}

pub fn canonical_task_event_order(events: &[LogicalTaskEvent]) -> Vec<LogicalTaskEvent> {
    let mut out = events.to_vec();
    out.sort_by(|lhs, rhs| {
        lhs.logical_time
            .cmp(&rhs.logical_time)
            .then_with(|| lhs.task_id.as_bytes().cmp(rhs.task_id.as_bytes()))
            .then_with(|| lhs.seq.cmp(&rhs.seq))
    });
    out
}

pub fn canonical_round_robin_order(task_ids: &[String], round: u64) -> Vec<String> {
    let mut out = task_ids
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<String>>();
    out.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    if !out.is_empty() {
        let len = out.len();
        out.rotate_left((round as usize) % len);
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskLifecycleDecision {
    Ready,
    TimedOut,
    Cancelled,
}

pub fn resolve_task_lifecycle(
    logical_tick: u64,
    deadline_tick: Option<u64>,
    cancelled: bool,
) -> TaskLifecycleDecision {
    if cancelled {
        return TaskLifecycleDecision::Cancelled;
    }
    if let Some(deadline) = deadline_tick {
        if logical_tick >= deadline {
            return TaskLifecycleDecision::TimedOut;
        }
    }
    TaskLifecycleDecision::Ready
}

pub fn canonicalize_text_boundary(raw: &[u8]) -> Result<String, DeterminismError> {
    let bytes = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &raw[3..]
    } else {
        raw
    };
    let text = std::str::from_utf8(bytes).map_err(|_| DeterminismError::InvalidUtf8)?;
    let linefeed = text.replace("\r\n", "\n").replace('\r', "\n");
    Ok(compose_unicode_subset(&linefeed))
}

pub fn preserve_runtime_literal(raw: &str) -> String {
    raw.to_string()
}

pub fn canonicalize_path_for_profile(
    raw: &str,
    profile: SupportedPlatformProfile,
) -> Result<String, DeterminismError> {
    normalize_path(raw, profile.case_sensitive())
}

pub fn canonicalize_iteration_paths(
    raw_paths: &[String],
    profile: SupportedPlatformProfile,
) -> Result<Vec<String>, DeterminismError> {
    let mut dedup = BTreeSet::<String>::new();
    for raw in raw_paths {
        dedup.insert(canonicalize_path_for_profile(raw, profile)?);
    }
    Ok(dedup.into_iter().collect::<Vec<String>>())
}

pub fn canonicalize_env_entries(entries: &[(String, String)]) -> Vec<(String, String)> {
    let mut out = entries.to_vec();
    out.sort_by(|lhs, rhs| {
        lhs.0
            .as_bytes()
            .cmp(rhs.0.as_bytes())
            .then_with(|| lhs.1.as_bytes().cmp(rhs.1.as_bytes()))
    });
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntropySource {
    HostRandom,
    Uuid,
    SystemTime,
    ProcessId,
    ThreadId,
    TempPath,
    GovernedSeededRng,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntropyPolicyDecision {
    pub allowed: bool,
    pub requires_audit_marker: bool,
    pub reason_code: &'static str,
}

pub fn evaluate_entropy_policy(
    lane: &str,
    source: EntropySource,
    governed_path: bool,
) -> EntropyPolicyDecision {
    match lane {
        "locked_v071" => {
            if source == EntropySource::GovernedSeededRng || governed_path {
                EntropyPolicyDecision {
                    allowed: true,
                    requires_audit_marker: false,
                    reason_code: "RC-ENTROPY-GOVERNED",
                }
            } else {
                EntropyPolicyDecision {
                    allowed: false,
                    requires_audit_marker: true,
                    reason_code: "RC-ENTROPY-BLOCKED-LOCKED",
                }
            }
        }
        "locked_v06" | "quarantine" => {
            if source == EntropySource::GovernedSeededRng || governed_path {
                EntropyPolicyDecision {
                    allowed: true,
                    requires_audit_marker: false,
                    reason_code: "RC-ENTROPY-GOVERNED",
                }
            } else {
                EntropyPolicyDecision {
                    allowed: true,
                    requires_audit_marker: true,
                    reason_code: "RC-ENTROPY-AUDIT",
                }
            }
        }
        _ => EntropyPolicyDecision {
            allowed: false,
            requires_audit_marker: true,
            reason_code: "RC-LANE-UNSUPPORTED",
        },
    }
}

fn normalize_path(raw: &str, case_sensitive: bool) -> Result<String, DeterminismError> {
    let mut normalized = raw.replace('\\', "/");
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }

    let absolute = normalized.starts_with('/');
    let mut stack = Vec::<String>::new();
    for part in normalized.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if stack.pop().is_none() {
                return Err(DeterminismError::PathEscape {
                    raw: raw.to_string(),
                });
            }
            continue;
        }
        let value = if case_sensitive {
            part.to_string()
        } else {
            part.to_ascii_lowercase()
        };
        stack.push(value);
    }

    let mut out = String::new();
    if absolute {
        out.push('/');
    }
    out.push_str(&stack.join("/"));
    if out.is_empty() {
        out.push('.');
    }
    Ok(out)
}

pub fn compare_bytes_lex(lhs: &str, rhs: &str) -> Ordering {
    lhs.as_bytes().cmp(rhs.as_bytes())
}

fn compose_unicode_subset(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let chars = raw.chars().collect::<Vec<char>>();
    let mut idx = 0usize;
    while idx < chars.len() {
        let current = chars[idx];
        let next = chars.get(idx + 1).copied();
        if let Some(combined) = compose_pair(current, next) {
            out.push(combined);
            idx += 2;
            continue;
        }
        out.push(current);
        idx += 1;
    }
    out
}

fn compose_pair(base: char, combining: Option<char>) -> Option<char> {
    match (base, combining) {
        ('a', Some('\u{0301}')) => Some('á'),
        ('A', Some('\u{0301}')) => Some('Á'),
        ('e', Some('\u{0301}')) => Some('é'),
        ('E', Some('\u{0301}')) => Some('É'),
        ('i', Some('\u{0301}')) => Some('í'),
        ('I', Some('\u{0301}')) => Some('Í'),
        ('o', Some('\u{0301}')) => Some('ó'),
        ('O', Some('\u{0301}')) => Some('Ó'),
        ('u', Some('\u{0301}')) => Some('ú'),
        ('U', Some('\u{0301}')) => Some('Ú'),
        ('y', Some('\u{0301}')) => Some('ý'),
        ('Y', Some('\u{0301}')) => Some('Ý'),
        ('a', Some('\u{0300}')) => Some('à'),
        ('A', Some('\u{0300}')) => Some('À'),
        ('e', Some('\u{0300}')) => Some('è'),
        ('E', Some('\u{0300}')) => Some('È'),
        ('i', Some('\u{0300}')) => Some('ì'),
        ('I', Some('\u{0300}')) => Some('Ì'),
        ('o', Some('\u{0300}')) => Some('ò'),
        ('O', Some('\u{0300}')) => Some('Ò'),
        ('u', Some('\u{0300}')) => Some('ù'),
        ('U', Some('\u{0300}')) => Some('Ù'),
        ('a', Some('\u{0303}')) => Some('ã'),
        ('A', Some('\u{0303}')) => Some('Ã'),
        ('n', Some('\u{0303}')) => Some('ñ'),
        ('N', Some('\u{0303}')) => Some('Ñ'),
        ('o', Some('\u{0303}')) => Some('õ'),
        ('O', Some('\u{0303}')) => Some('Õ'),
        ('a', Some('\u{0302}')) => Some('â'),
        ('A', Some('\u{0302}')) => Some('Â'),
        ('e', Some('\u{0302}')) => Some('ê'),
        ('E', Some('\u{0302}')) => Some('Ê'),
        ('i', Some('\u{0302}')) => Some('î'),
        ('I', Some('\u{0302}')) => Some('Î'),
        ('o', Some('\u{0302}')) => Some('ô'),
        ('O', Some('\u{0302}')) => Some('Ô'),
        ('u', Some('\u{0302}')) => Some('û'),
        ('U', Some('\u{0302}')) => Some('Û'),
        ('a', Some('\u{0308}')) => Some('ä'),
        ('A', Some('\u{0308}')) => Some('Ä'),
        ('e', Some('\u{0308}')) => Some('ë'),
        ('E', Some('\u{0308}')) => Some('Ë'),
        ('i', Some('\u{0308}')) => Some('ï'),
        ('I', Some('\u{0308}')) => Some('Ï'),
        ('o', Some('\u{0308}')) => Some('ö'),
        ('O', Some('\u{0308}')) => Some('Ö'),
        ('u', Some('\u{0308}')) => Some('ü'),
        ('U', Some('\u{0308}')) => Some('Ü'),
        ('c', Some('\u{0327}')) => Some('ç'),
        ('C', Some('\u{0327}')) => Some('Ç'),
        _ => None,
    }
}
