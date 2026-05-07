mod ast;
mod lexer;
mod parser;
mod runtime;
mod drivers;
mod browser;
mod email;
mod sys;
mod ui;

use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "e", version = "5.0.0", about = "E — general-purpose language")]
struct Cli {
    /// Script file to run
    file: PathBuf,

    /// Live execution mode
    #[arg(long)]
    live: bool,

    /// Keep alive for scheduled tasks
    #[arg(long)]
    watch: bool,

    /// Arguments passed to the E script
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    script_args: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    let filepath = cli.file.to_string_lossy().to_string();

    let source = match fs::read_to_string(&cli.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ could not read '{}': {}", cli.file.display(), e);
            std::process::exit(1);
        }
    };

    // Set up panic hook to print user-friendly error instead of raw panic
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            format!("{:?}", info.payload())
        };
        eprintln!("❌ {}", msg);
    }));

    let has_sections = source.contains("\n:sys") || source.contains("\n:core") || source.contains("\n:ui")
        || source.starts_with(":sys") || source.starts_with(":core") || source.starts_with(":ui");

    let script_args = cli.script_args;
    let watch = cli.watch;
    let live = cli.live;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if has_sections {
            run_eee(&filepath, &source, live);
        } else {
            run_e(&filepath, &source, live, watch, &script_args);
        }
    }));

    // Restore original hook
    let _ = std::panic::take_hook();
    std::panic::set_hook(orig_hook);

    match result {
        Ok(()) => {},
        Err(_) => {
            eprintln!("\n❌ Execution failed. Check the error above.");
            std::process::exit(1);
        }
    }
}

fn run_e(filepath: &str, source: &str, live: bool, watch: bool, script_args: &[String]) {
    let tokens = match lexer::lex(source) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ lex error: {}", e);
            std::process::exit(1);
        }
    };

    let ast = {
        let mut p = parser::Parser::new(tokens);
        p.parse()
    };

    println!("▶️  E — {} ({})", filepath, if live { "live" } else { "dry-run" });
    println!("{}", "=".repeat(60));

    if live {
        let mut driver = drivers::RealDriver::with_watch(watch);
        let mut scope = runtime::Scope::new();
        set_args(&mut scope, script_args, filepath);
        for node in &ast {
            runtime::exec_node(node, &mut scope, &mut driver);
        }
    } else {
        let mut driver = drivers::DryDriver::with_watch(watch);
        let mut scope = runtime::Scope::new();
        set_args(&mut scope, script_args, filepath);
        for node in &ast {
            runtime::exec_node(node, &mut scope, &mut driver);
        }
    }

    println!("\n✅ Done: {}", filepath);
}

fn set_args(scope: &mut runtime::Scope, args: &[String], script: &str) {
    use crate::ast::Value;
    let mut all = vec![Value::Str(script.to_string())];
    for a in args {
        all.push(Value::Str(a.clone()));
    }
    scope.def_var("args", Value::List(all));
}

fn run_eee(filepath: &str, source: &str, live: bool) {
    // Note: --watch for 3-tier files is handled by the core section's TimeNode
    // run_eee doesn't use script_args currently — args are only for non-3-tier scripts
    let efile = match parser::parse_eee(source) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("❌ parse error: {}", e);
            std::process::exit(1);
        }
    };

    println!("▶️  E — {} (3-tier)", filepath);
    println!("{}", "=".repeat(60));

    // Handle :sys section — load plugins
    // Plugins are now auto-registered by RealDriver on creation.
    // The :sys section is informational for the user.
    if let Some(ref sys) = efile.sys_section {
        for line in sys.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") {
                let path = trimmed[4..].trim().trim_matches('"');
                let plugin_name = std::path::Path::new(path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.trim_start_matches("lib").trim_end_matches(".eso").to_string())
                    .unwrap_or_else(|| path.to_string());
                println!("  🔌 loaded plugin: {}", plugin_name);
            }
        }
    }

    // Handle :core section — execute E code FIRST
    if live {
        let mut driver = drivers::RealDriver::new();
        let mut scope = runtime::Scope::new();
        for node in &efile.core_section {
            runtime::exec_node(node, &mut scope, &mut driver);
        }
    } else {
        let mut driver = drivers::DryDriver::new();
        let mut scope = runtime::Scope::new();
        for node in &efile.core_section {
            runtime::exec_node(node, &mut scope, &mut driver);
        }
    }

    // Handle :ui section — open window AFTER core
    if let Some(ref ui) = efile.ui_section {
        println!("  🖥️ Opening UI window...");
        if live {
            let _ = ui::Ui::open_window(ui);
        }
    }

    println!("\n✅ Done: {}", filepath);
}
