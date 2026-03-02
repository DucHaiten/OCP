use std::env;

fn parse_inputs(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == "--inputs" && i + 1 < args.len() {
            out.push(args[i + 1].clone());
            i += 2;
            continue;
        }
        i += 1;
    }
    out
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let inputs = parse_inputs(&args);
    if inputs.is_empty() {
        println!("soak_compare: no inputs provided");
    } else {
        println!("soak_compare: {} input set(s)", inputs.len());
    }
}

#[cfg(test)]
mod tests {
    use super::parse_inputs;

    #[test]
    fn parse_inputs_extracts_values() {
        let args = vec![
            "--inputs".to_string(),
            "artifacts/a".to_string(),
            "--format".to_string(),
            "table".to_string(),
            "--inputs".to_string(),
            "artifacts/b".to_string(),
        ];
        let inputs = parse_inputs(&args);
        assert_eq!(
            inputs,
            vec!["artifacts/a".to_string(), "artifacts/b".to_string()]
        );
    }
}
