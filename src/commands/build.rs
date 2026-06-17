use color_eyre::eyre;
use rust_embed::Embed;
use subprocess::{Exec, Redirection};
use wasmparser::Encoding;
use std::env;
use std::path::{Path, PathBuf};

use toml::Table;

use wit_component::ComponentEncoder;

use crate::Result;

#[derive(Embed)]
#[folder = "wasi"]
struct WasiWasm;

/// Prebuilt web runtime artifacts assembled into a `--target web` site:
/// the wasm-bindgen host (`web.js` + `web_bg.wasm`), the HTML/JS harness, and the
/// preview2 WASI browser shim. Regenerate with `scripts/build-web-runtime.sh`.
#[derive(Embed)]
#[folder = "web-runtime"]
struct WebRuntime;

fn read_config() -> Result<Table> {
    Ok(std::fs::read_to_string("jumpjet.toml")?.parse::<Table>()?)
}

/// Runs the `[build].pre` command from `jumpjet.toml` (e.g. `cargo build --target
/// wasm32-wasip2`), if present.
fn run_pre(config: &Table) -> Result<()> {
    let build = config.get("build").unwrap();
    if let Some(command) = build.get("pre") {
        let result = Exec::shell(command.as_str().unwrap())
            .stdout(Redirection::Pipe)
            .stderr(Redirection::Merge)
            .capture()
            .expect("pre command execution failed");

        println!("{}", result.stdout_str());

        if !result.success() {
            return Err(eyre::eyre!("pre command execution failed"));
        }
    }
    Ok(())
}

/// Copies the build `input` dir into the `output` dir (`bin/`) and componentizes the
/// entrypoint in place, returning the path to the componentized wasm.
fn componentize_input(config: &Table) -> Result<PathBuf> {
    let current_dir = env::current_dir()?;

    let entrypoint = config["build"]["entrypoint"]
        .as_str()
        .expect("No build entrypoint provided in config!");
    let input_path = config["build"]["input"]
        .as_str()
        .expect("No build input provided in config!");
    let input_path = Path::new(input_path);
    let output_path = Path::new(config["build"]["output"].as_str().unwrap_or("bin"));

    crate::fs::copy_dir_all(input_path, output_path)?;

    // TODO: Concatenate jumpjet dependencies read from config to wasm binary
    let output_entrypoint_path = current_dir.join(output_path).join(entrypoint);
    componentize_wasm(output_entrypoint_path.clone());

    Ok(output_entrypoint_path)
}

/// The directory `build --target web` transpiles the guest into (`<output>/web/guest`).
fn web_guest_dir(config: &Table) -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    let output_path = Path::new(config["build"]["output"].as_str().unwrap_or("bin"));
    Ok(current_dir.join(output_path).join("web").join("guest"))
}

/// The build input artifact `run --target web` watches: `<input>/<entrypoint>`.
pub fn input_entrypoint(config: &Table) -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    let entrypoint = config["build"]["entrypoint"]
        .as_str()
        .expect("No build entrypoint provided in config!");
    let input_path = config["build"]["input"]
        .as_str()
        .expect("No build input provided in config!");
    Ok(current_dir.join(input_path).join(entrypoint))
}

/// The native build: run the `pre` command, then componentize the guest into `bin/`.
pub async fn build(_release: &bool) -> Result<()> {
    let config = read_config()?;
    run_pre(&config)?;
    componentize_input(&config)?;
    Ok(())
}

/// Compiles the guest for the web: `pre` + componentize + `jco transpile` into
/// `<output>/web/guest`. Does NOT emit the host runtime / harness / page — that is
/// the assembly step shared by `bundle --target web` and `run --target web`
/// (see [`assemble_web_site`]).
pub async fn build_web(release: &bool) -> Result<()> {
    let config = read_config()?;
    run_pre(&config)?;
    build_web_compile(&config)?;
    println!("Web guest compiled to {}", web_guest_dir(&config)?.display());
    let _ = release;
    Ok(())
}

/// Like [`build_web`] but skips the `pre` command. Used by the `run --target web`
/// watch loop, which reacts to the developer's own compiler output rather than
/// invoking the guest compiler itself.
pub async fn build_web_incremental(_release: &bool) -> Result<()> {
    let config = read_config()?;
    build_web_compile(&config)?;
    Ok(())
}

/// Shared compile step: componentize the input + transpile the guest to JS.
fn build_web_compile(config: &Table) -> Result<()> {
    let component_path = componentize_input(config)?;
    let guest_dir = web_guest_dir(config)?;
    transpile_guest(&component_path, &guest_dir)
}

/// Transpiles the componentized guest to JS via `jco` into `guest_dir`.
fn transpile_guest(component_path: &Path, guest_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(guest_dir)?;

    // jco transpile (instantiation mode) -> guest/guest.js + core wasm.
    let capture = Exec::cmd("jco")
        .arg("transpile")
        .arg(component_path)
        .args(&["--instantiation", "async", "--name", "guest", "-o"])
        .arg(guest_dir)
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
        return Err(eyre::eyre!("jco transpile failed:\n{}", capture.stdout_str()));
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

    Ok(())
}

/// Writes the embedded host runtime + harness + page + WASI shim into `out_dir`.
pub fn write_web_runtime(out_dir: &Path) -> Result<()> {
    for file in WebRuntime::iter() {
        let rel = file.as_ref();
        let data = WebRuntime::get(rel)
            .ok_or_else(|| eyre::eyre!("missing embedded web-runtime file: {rel}"))?;
        let dest = out_dir.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(dest, data.data)?;
    }
    Ok(())
}

/// Assembles a complete, servable web site at `out_dir`: the transpiled guest
/// (`guest_dir`, copied to `out_dir/guest` unless already there) plus the embedded
/// host runtime / harness / page. Shared by `bundle --target web` and
/// `run --target web`.
pub fn assemble_web_site(guest_dir: &Path, out_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(out_dir)?;
    let dest_guest = out_dir.join("guest");
    if guest_dir != dest_guest {
        crate::fs::copy_dir_all(guest_dir, &dest_guest)?;
    }
    write_web_runtime(out_dir)?;
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
