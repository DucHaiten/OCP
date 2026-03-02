#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRef {
    pub raw: String,
    pub family: String,
    pub parts: Vec<String>,
}

pub fn parse_key(input: &str) -> Option<KeyRef> {
    let raw = input.trim();
    if raw.is_empty() {
        return None;
    }

    let parts: Vec<String> = raw.split('.').map(|s| s.trim().to_string()).collect();
    if parts.len() < 2 {
        return None;
    }
    if parts.iter().any(|p| p.is_empty()) {
        return None;
    }
    if parts.iter().any(|p| !is_key_part_valid(p)) {
        return None;
    }

    Some(KeyRef {
        raw: raw.to_string(),
        family: parts[0].clone(),
        parts,
    })
}

fn is_key_part_valid(part: &str) -> bool {
    part.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}
