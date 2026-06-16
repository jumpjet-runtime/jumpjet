#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use std::env;
    use std::fs;
    use jumpjet::runtime;
    use clap::Parser;

    #[derive(Parser)]
    struct Cli {
        entrypoint: String,
        #[clap(short, long)]
        debug: bool,
    }

    let args = Cli::parse();

    let entrypoint = args.entrypoint;
    let input_path = env::current_exe().unwrap().parent().unwrap().join(format!(".jumpjet/input/{entrypoint}"));
    let binary = fs::read(&input_path).expect("Failed to read the WASM file");
    runtime::run(input_path, binary, args.debug);
}

// The `native` bin is not used on wasm (the web build uses the `web` cdylib),
// but Cargo still compiles every bin target for the wasm32 build.
#[cfg(target_arch = "wasm32")]
fn main() {}
