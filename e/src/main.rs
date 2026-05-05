mod ast;
mod lexer;
mod parser;
mod runtime;
mod drivers;

use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "e", version = "2.0.0", about = "E — general-purpose language")]
struct Cli {
    /// Script file to run
    file: PathBuf,

    /// Live execution mode
    #[arg(long)]
    live: bool,
}

fn main() {
    let cli = Cli::parse();

    let source = match fs::read_to_string(&cli.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ could not read '{}': {}", cli.file.display(), e);
            std::process::exit(1);
        }
    };

    let tokens = match lexer::lex(&source) {
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

    println!("▶️  E — {} (dry-run)", cli.file.display());
    println!("{}", "=".repeat(60));

    if cli.live {
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

    println!("\n✅ Done: {}", cli.file.display());
}
