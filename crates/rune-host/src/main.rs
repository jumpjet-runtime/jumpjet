use std::env;
use std::fs;
use rune::runtime;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    entrypoint: String,
    #[clap(short, long)]
    debug: bool,
}

fn main() {
    let args = Cli::parse();

    let entrypoint = args.entrypoint;
    let input_path = env::current_exe().unwrap().parent().unwrap().join(format!(".rune/input/{entrypoint}"));
    let binary = fs::read(&input_path).expect("Failed to read the WASM file");
    runtime::run(input_path, binary, args.debug);
}
