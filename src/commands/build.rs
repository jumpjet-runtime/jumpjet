use color_eyre::eyre;
use rust_embed::Embed;
use subprocess::{Exec, ExitStatus, Redirection};
use wasmparser::{Encoding, Payload};
use std::env;
use std::path::{Path, PathBuf};

use toml::Table;

use wit_component::{
    ComponentEncoder, DecodedWasm, Linker, StringEncoding, WitPrinter,
};

use crate::cli::NewSubcommand;

use crate::Result;

#[derive(Embed)]
#[folder = "wasi"]
struct WasiWasm;

/// Prebuilt web runtime artifacts emitted alongside a `--target web` build:
/// the wasm-bindgen host (`web.js` + `web_bg.wasm`), the HTML/JS harness, and the
/// preview2 WASI browser shim. Regenerate with `scripts/build-web-runtime.sh`.
#[derive(Embed)]
#[folder = "web-runtime"]
struct WebRuntime;

pub async fn build(release: &bool) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config = std::fs::read_to_string("rune.toml")
        .unwrap()
        .parse::<Table>()
        .unwrap();

    let build = config.get("build").unwrap();

    let pre_command = build.get("pre");
    if let Some(command) = pre_command {
        let result = Exec::shell(command.as_str().unwrap())
            .stdout(Redirection::Pipe) 
            .stderr(Redirection::Merge)
            .capture()
            .expect("pre command execution failed");

        let stdout = result.stdout_str();
        println!("{}", stdout);

        if !result.success() {
            return Err(eyre::eyre!("pre command execution failed"));
        }
    }

    let entrypoint = match config["build"]["entrypoint"].as_str() {
        Some(entrypoint) => entrypoint,
        None => panic!("No build input provided in config!"),
    };

    let input_path = config["build"]["input"].as_str();
    if input_path.is_none() {
        panic!("No build input provided in config!")
    }
    let input_path = Path::new(input_path.unwrap());
    let entrypoint_path = input_path.join(entrypoint);
    let binary = std::fs::read(&entrypoint_path).unwrap();

    let output_path = Path::new(config["build"]["output"].as_str().unwrap_or("bin"));

    crate::fs::copy_dir_all(input_path, output_path)?;

    let output_entrypoint_path = current_dir.join(&output_path).join(&entrypoint);

    // TODO: Concatenate rune dependencies read from config to wasm binary
    
    componentize_wasm(output_entrypoint_path);

    Ok(())
}

/// Builds the project, then emits a runnable web bundle next to the native
/// output (`<output>/web`). The guest component is transpiled to JS via jco and
/// combined with the embedded host runtime + harness + WASI shim.
pub async fn build_web(release: &bool) -> Result<()> {
    build(release).await?;

    let current_dir = env::current_dir()?;
    let config = std::fs::read_to_string("rune.toml")
        .unwrap()
        .parse::<Table>()
        .unwrap();

    let entrypoint = config["build"]["entrypoint"]
        .as_str()
        .expect("No build entrypoint provided in config!");
    let output_path = Path::new(config["build"]["output"].as_str().unwrap_or("bin"));
    let component_path = current_dir.join(output_path).join(entrypoint);
    let web_out = current_dir.join(output_path).join("web");

    emit_web_bundle(&component_path, &web_out)?;

    println!("Web build emitted to {}", web_out.display());
    println!("Serve it over HTTP (e.g. `python3 -m http.server` from that dir) and open in a WebGPU-capable browser.");
    Ok(())
}

/// Transpiles the guest component with jco and writes the embedded web runtime
/// artifacts into `web_out`.
fn emit_web_bundle(component_path: &Path, web_out: &Path) -> Result<()> {
    let guest_dir = web_out.join("guest");
    std::fs::create_dir_all(&guest_dir)?;

    // jco transpile (instantiation mode) -> guest/guest.js + core wasm.
    let capture = Exec::cmd("jco")
        .arg("transpile")
        .arg(component_path)
        .args(&["--instantiation", "async", "--name", "guest", "-o"])
        .arg(&guest_dir)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .capture();

    let capture = match capture {
        Ok(c) => c,
        Err(_) => {
            return Err(eyre::eyre!(
                "`jco` was not found on your PATH. Install it with `npm i -g @bytecodealliance/jco`."
            ))
        }
    };
    if !capture.success() {
        return Err(eyre::eyre!(
            "jco transpile failed:\n{}",
            capture.stdout_str()
        ));
    }

    // Workaround for a jco 1.23 code-gen bug: resource-method trampolines
    // reference an undeclared `currentSubtask` inside (no-op) debug-log calls,
    // which throws a ReferenceError that masks real errors. A module-scope
    // declaration makes those resolve to `undefined` harmlessly.
    let guest_js = guest_dir.join("guest.js");
    if let Ok(src) = std::fs::read_to_string(&guest_js) {
        if !src.starts_with("var currentSubtask;") {
            std::fs::write(&guest_js, format!("var currentSubtask;\n{src}"))?;
        }
    }

    // Emit the embedded host runtime, harness, and WASI shim.
    for file in WebRuntime::iter() {
        let rel = file.as_ref();
        let data = WebRuntime::get(rel)
            .ok_or_else(|| eyre::eyre!("missing embedded web-runtime file: {rel}"))?;
        let dest = web_out.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(dest, data.data)?;
    }

    Ok(())
}

fn componentize_wasm(output_entrypoint_path: PathBuf) {
    let parser = wat::Parser::new();
    let wasm = parser.parse_file(&output_entrypoint_path).expect("Unable to read game wasm");
    let mut encoder = ComponentEncoder::default()
        .validate(true)
        .reject_legacy_names(false);

    let bytes: Vec<u8>;
    let mut is_component = false;
    for payload in wasmparser::Parser::new(0).parse_all(&wasm) {
        let payload = payload.expect("No wasm payload");
        match payload {
            wasmparser::Payload::Version { encoding, .. } if encoding != Encoding::Module => {
                is_component = true;
            }
            _ => { }
        }
    }

    if is_component {
        bytes = wasm;
    } else {
        // encoder = encoder.merge_imports_based_on_semver(merge); // TODO: Needed?
        encoder = encoder.module(&wasm).expect("Unable to read game as a wasm module");

        let adapter = WasiWasm::get("wasi_snapshot_preview1.reactor.wasm").unwrap();
        let adapter = wat::parse_bytes(&adapter.data).unwrap();
        encoder = encoder.adapter("wasi_snapshot_preview1", &adapter).expect("Unable to read adapter");

        bytes = encoder
            .encode()
            .expect("Failed to encode a component from provided module");
    }

    std::fs::write(&output_entrypoint_path, bytes).expect("Unable to write wasm");
}
