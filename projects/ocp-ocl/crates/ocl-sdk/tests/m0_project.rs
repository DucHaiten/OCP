use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{
    build_project, check_project, fmt_project, init_project, run_project, run_reactor_ticks,
    test_project,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_m0_{tag}_{stamp}"))
}

#[test]
fn m0_sdk_init_check_run_fmt_test_build_pass() {
    let root = temp_project_dir("full");
    init_project(&root).expect("init");

    let check = check_project(&root).expect("check");
    assert!(check.files_checked >= 1);

    let run = run_project(&root).expect("run");
    assert!(run.steps > 0);

    let main_file = root.join("src").join("main.ocl");
    fs::write(
        &main_file,
        "fn on_event(ev) { return ev; }\nlet ok = true;\ncondition(ok);\n",
    )
    .expect("rewrite main");
    let reactor = run_reactor_ticks(&root, 4).expect("reactor");
    assert_eq!(reactor.ticks, 4);
    assert!(reactor.total_steps > 0);

    let fmt = fmt_project(&root, false).expect("fmt");
    assert_eq!(fmt.files_touched, 0);

    let tests = test_project(&root).expect("test");
    assert!(tests.tests_run >= 1);

    let build = build_project(&root).expect("build");
    assert!(build.files_bundled >= 1);
}
