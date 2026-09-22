//! Entry point for the SUMER compiler CLI binary (`sumer`).

use std::env;
use std::io;
use std::process;

use sumer_cli::run_cli;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let exit_code = run_cli(&args, &mut stdout, &mut stderr);
    if exit_code != 0 {
        process::exit(exit_code);
    }
}
