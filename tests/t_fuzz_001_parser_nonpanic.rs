use std::panic;

use ocp::ocp::parse_program;

fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn gen_program_fragment(state: &mut u64) -> String {
    let templates = [
        "let a = true;\n",
        "let b = 1;\n",
        "condition(true);\n",
        "condition(false);\n",
        "repeat 2 { let z = true; }\n",
        "let xs = [1, 2, 3];\n",
        "for item in xs cap 2 { let y = item; }\n",
        "observe(\"world.exists\", \"tier2\", ctx(\"scene=lab\"), budget(5)) -> r;\n",
        "match r {\n  OK => { let m = 1; }\n  DEGRADED => { let m = 2; }\n  INSUFFICIENT => { let m = 3; }\n  DEFERRED => { let m = 4; }\n}\n",
        "entangle(a, b, true);\n",
        "guard r;\n",
        "let q = try r else { r.reason_code };\n",
    ];

    let picks = (xorshift64(state) % 6 + 1) as usize;
    let mut out = String::new();
    for _ in 0..picks {
        let idx = (xorshift64(state) % templates.len() as u64) as usize;
        out.push_str(templates[idx]);
    }
    if !out.contains("condition(") {
        out.push_str("condition(true);\n");
    }
    out
}

#[test]
fn t_fuzz_001_parser_nonpanic_seeded() {
    let seeds = [11_u64, 17, 23, 47, 89];
    let cases_per_seed = 400_u32;

    for seed in seeds {
        let mut state = seed;
        for case_idx in 0..cases_per_seed {
            let src = gen_program_fragment(&mut state);
            let file_id = seed as u32 + case_idx + 1;

            let result = panic::catch_unwind(|| parse_program(&src, file_id));
            assert!(
                result.is_ok(),
                "parser panicked at seed={seed}, case={case_idx}, source=\n{src}"
            );

            if let Ok(Err(diag)) = result {
                assert!(
                    !diag.code.as_str().is_empty(),
                    "diagnostic code must be non-empty at seed={seed}, case={case_idx}"
                );
            }
        }
    }
}
