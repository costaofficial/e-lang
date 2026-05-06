mod ast;
mod lexer;
mod parser;
mod runtime;
mod drivers;
mod browser;
mod email;
mod sys;

use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "e", version = "2.1.0", about = "E — general-purpose language")]
struct Cli {
    /// Script file to run
    file: PathBuf,

    /// Live execution mode
    #[arg(long)]
    live: bool,
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

    // Check if this is an .eee file with sections
    let has_sections = source.contains("\n:sys") || source.contains("\n:core") || source.contains("\n:ui")
        || source.starts_with(":sys") || source.starts_with(":core") || source.starts_with(":ui");

    if has_sections {
        run_eee(&filepath, &source, cli.live);
    } else {
        run_e(&filepath, &source, cli.live);
    }
}

fn run_e(filepath: &str, source: &str, live: bool) {
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

    println!("▶️  E — {} (dry-run)", filepath);
    println!("{}", "=".repeat(60));

    if live {
        let mut driver = drivers::RealDriver::new();
        let mut scope = runtime::Scope::new();
        for node in &ast {
            runtime::exec_node(node, &mut scope, &mut driver);
        }
    } else {
        let mut driver = drivers::DryDriver::new();
        let mut scope = runtime::Scope::new();
        for node in &ast {
            runtime::exec_node(node, &mut scope, &mut driver);
        }
    }

    println!("\n✅ Done: {}", filepath);
}

fn run_eee(filepath: &str, source: &str, live: bool) {
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
    if let Some(ref sys) = efile.sys_section {
        let mut pm = runtime::PLUGIN_MANAGER.lock().unwrap();
        for line in sys.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") {
                let path = trimmed[4..].trim().trim_matches('"');
                match pm.load(path) {
                    Ok(_) => println!("  🔌 loaded plugin: {}", path),
                    Err(e) => eprintln!("  ⚠️ {}", e),
                }
            }
        }
    }

    // Handle :ui section
    if let Some(ref ui) = efile.ui_section {
        println!("  🖥️ UI section ({} bytes)", ui.len());
    }

    // Handle :core section — execute E code
    let mode_live = live;
    if mode_live {
        let mut driver = drivers::RealDriver::new();
        let mut scope = runtime::Scope::new();
        // Register sys.call function in scope
        scope.def_fn("sys.call", vec!["plugin".into(), "func".into(), "args".into()], vec![]);
        for node in &efile.core_section {
            runtime::exec_node(node, &mut scope, &mut driver);
        }
    } else {
        let mut driver = drivers::DryDriver::new();
        let mut scope = runtime::Scope::new();
        scope.def_fn("sys.call", vec!["plugin".into(), "func".into(), "args".into()], vec![]);
        for node in &efile.core_section {
            runtime::exec_node(node, &mut scope, &mut driver);
        }
    }

    println!("\n✅ Done: {}", filepath);
}
