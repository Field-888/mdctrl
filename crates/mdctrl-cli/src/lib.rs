//! Command-line client for mdctrl.

use std::process::ExitCode;

pub fn run() -> ExitCode {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("--version" | "-V") => {
            println!("mdctrl {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") => {
            print_help();
            ExitCode::SUCCESS
        }
        Some(argument) => {
            eprintln!("unknown argument: {argument}");
            print_help();
            ExitCode::FAILURE
        }
        None => {
            print_help();
            ExitCode::SUCCESS
        }
    }
}

fn print_help() {
    println!("mdctrl {}", env!("CARGO_PKG_VERSION"));
    println!("Usage: mdctrl [--version] [--help]");
}
