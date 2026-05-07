use std::process::Command;
use std::path::PathBuf;

fn get_e_path() -> PathBuf {
    // When running tests via `cargo test`, the CWD is the crate root (e/)
    // The binary is at e/target/debug/e
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    if path.ends_with("deps") {
        path.pop(); // remove deps/
    }
    path.push("e");
    path
}

#[test]
fn test_core_suite() {
    let e_path = get_e_path();
    let mut examples = std::env::current_dir().unwrap();
    examples.push("examples");
    examples.push("core.eee");

    let output = Command::new(&e_path)
        .arg(examples.to_str().unwrap())
        .output()
        .expect("failed to run e");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ALL TESTS PASSED"),
        "Core tests failed!\nstdout:\n{}", stdout);
}

#[test]
fn test_plugins_suite() {
    let e_path = get_e_path();
    let mut examples = std::env::current_dir().unwrap();
    examples.push("examples");
    examples.push("plugins.eee");

    let output = Command::new(&e_path)
        .arg("--live")
        .arg(examples.to_str().unwrap())
        .output()
        .expect("failed to run e");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ALL TESTS PASSED"),
        "Plugin tests failed!\nstdout:\n{}", stdout);
}
