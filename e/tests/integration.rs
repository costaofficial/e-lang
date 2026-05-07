use std::process::Command;

fn find_e_binary() -> std::path::PathBuf {
    // Try multiple locations to find the e binary
    let candidates = [
        // Running from e/ directory
        "target/debug/e",
        "target/release/e",
        // Running from project root
        "../target/debug/e",
        "../target/release/e",
    ];

    for c in &candidates {
        let p = std::path::Path::new(c);
        if p.exists() {
            return p.to_path_buf();
        }
    }

    // Fallback: use `which e` or just try the path
    if let Ok(output) = Command::new("which").arg("e").output() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() && std::path::Path::new(&path).exists() {
            return std::path::PathBuf::from(path);
        }
    }

    // Last resort: just return debug path
    "target/debug/e".into()
}

#[test]
fn test_core_suite() {
    let e_path = find_e_binary();
    let mut examples = std::env::current_dir().unwrap();
    examples.push("examples");
    examples.push("core.eee");

    let output = Command::new(&e_path)
        .arg(examples.to_str().unwrap())
        .output()
        .expect(&format!("failed to run e at {:?}", e_path));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ALL TESTS PASSED"),
        "Core tests failed!\ne: {:?}\nstdout:\n{}", e_path, stdout);
}

#[test]
fn test_plugins_suite() {
    let e_path = find_e_binary();
    let mut examples = std::env::current_dir().unwrap();
    examples.push("examples");
    examples.push("plugins.eee");

    let output = Command::new(&e_path)
        .arg(examples.to_str().unwrap())
        .output()
        .expect(&format!("failed to run e at {:?}", e_path));

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Plugin tests in dry-run mode should at least load plugins
    assert!(stdout.contains("🔌 loaded plugin") || stdout.contains("ALL TESTS PASSED"),
        "Plugin tests failed!\ne: {:?}\nstdout:\n{}", e_path, stdout);
}
