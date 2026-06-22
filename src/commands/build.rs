use color_eyre::eyre;
use rust_embed::Embed;
use std::env;
use std::path::{Path, PathBuf};
use subprocess::{Exec, Redirection};
use wasmparser::Encoding;

use wit_component::ComponentEncoder;

use crate::Result;
use crate::pkg::compose::ComposeDep;
use crate::pkg::manifest::{Build, Manifest};
use crate::pkg::resolve::Resolution;

#[derive(Embed)]
#[folder = "wasi"]
struct WasiWasm;

/// The fixed filename the componentized guest is written to inside the build
/// `output` dir (and shipped under in every bundle). Using a canonical name means
/// the runtime and generated wrappers never need to know the source wasm's name.
pub const ENTRYPOINT_FILE: &str = "entrypoint.wasm";

/// The fixed filename the componentized headless server is written to inside the
/// build `output` dir, alongside the client's [`ENTRYPOINT_FILE`].
pub const SERVER_FILE: &str = "server.wasm";

/// Prebuilt web runtime artifacts assembled into a `--target web` site:
/// the wasm-bindgen host (`web.js` + `web_bg.wasm`), the HTML/JS harness, and the
/// preview2 WASI browser shim. Regenerate with `scripts/build-web-runtime.sh`.
#[derive(Embed)]
#[folder = "web-runtime"]
struct WebRuntime;

/// Runs a component's `pre` command (e.g. `cargo build --target wasm32-wasip2`),
/// if present.
fn run_pre(build: &Build) -> Result<()> {
    if let Some(command) = &build.pre {
        let result = Exec::shell(command)
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

/// Builds the shippable `output` dir (`bin/`): copy the optional `assets` tree in,
/// then componentize the built `entrypoint` wasm into `output/<out_name>` (in
/// place). Only these — the component(s) plus declared assets — ever enter a
/// bundle, so build-tool cruft (cargo target internals, JS dist intermediates)
/// stays out. Returns the path to the componentized wasm.
fn componentize(build: &Build, out_name: &str) -> Result<PathBuf> {
    let current_dir = env::current_dir()?;

    let entrypoint = build
        .entrypoint
        .as_deref()
        .ok_or_else(|| eyre::eyre!("no build entrypoint provided in jumpjet.toml"))?;
    let output_path = current_dir.join(build.output.as_deref().unwrap_or("bin"));

    std::fs::create_dir_all(&output_path)?;

    // Optional data files the game ships, mounted as its local storage at runtime.
    // Copied before the entrypoint so the canonical wasm name always wins.
    if let Some(assets) = build.assets.as_deref() {
        let assets_dir = current_dir.join(assets);
        if assets_dir.exists() {
            crate::fs::copy_dir_all(&assets_dir, &output_path)?;
        }
    }

    // Package dependencies are composed in afterwards by `compose_into`.
    let output_entrypoint_path = output_path.join(out_name);
    std::fs::copy(current_dir.join(entrypoint), &output_entrypoint_path)?;
    componentize_wasm(output_entrypoint_path.clone());

    Ok(output_entrypoint_path)
}

/// The directory `build --target web` transpiles the guest into (`<output>/web/guest`).
fn web_guest_dir(build: &Build) -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    let output_path = Path::new(build.output.as_deref().unwrap_or("bin"));
    Ok(current_dir.join(output_path).join("web").join("guest"))
}

/// The built wasm artifact `run --target web` watches: the `entrypoint` path.
pub fn source_entrypoint(build: &Build) -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    let entrypoint = build
        .entrypoint
        .as_deref()
        .ok_or_else(|| eyre::eyre!("no build entrypoint provided in jumpjet.toml"))?;
    Ok(current_dir.join(entrypoint))
}

/// The native build: run the `pre` command, then componentize the guest into `bin/`.
/// For `type = "lib"` packages, also emit the package's WIT alongside the component
/// and validate that it only imports Jumpjet/WASI host APIs.
pub async fn build(_release: &bool) -> Result<()> {
    let manifest = Manifest::load()?;
    // Resolve + stage dependency WIT *before* `pre`, so guest bindgen can see it.
    let resolution = prepare_deps().await?;

    // Primary component: the game/client (or the lib).
    let primary = manifest.primary_build()?;
    run_pre(primary)?;
    let component_path = componentize(primary, ENTRYPOINT_FILE)?;
    // Compose dependency components into the freshly built guest component.
    compose_into(&component_path, &resolution)?;

    if manifest.is_lib() {
        finalize_lib(&manifest, &component_path)?;
    }

    // Optional headless server component (multiplayer). Built into the same
    // `output` dir as `server.wasm`; dependency composition for the server is a
    // follow-up (servers are dependency-free for now).
    if let Some(server) = manifest.server_build() {
        run_pre(server)?;
        componentize(server, SERVER_FILE)?;
    }

    Ok(())
}

/// Resolves the project's `[dependencies]` and stages their WIT into the source
/// tree. Returns the resolution so the caller can compose the components in after
/// the guest is componentized. A no-op for projects without dependencies.
async fn prepare_deps() -> Result<Resolution> {
    let dir = env::current_dir()?;
    let manifest = Manifest::load_from(&dir)?;
    let resolution = crate::pkg::resolve::resolve(&dir, false).await?;
    crate::pkg::stage::stage_wit(&dir, &manifest, &resolution)?;
    Ok(resolution)
}

/// Composes the resolved dependency components into the guest component at
/// `component_path`, writing the result back in place. A no-op without deps.
fn compose_into(component_path: &Path, resolution: &Resolution) -> Result<()> {
    if resolution.packages.is_empty() {
        return Ok(());
    }
    let consumer = std::fs::read(component_path)?;
    let deps = resolution
        .packages
        .iter()
        .map(|p| {
            Ok(ComposeDep {
                id: p.id.clone(),
                component: p.stored.read_component()?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let composed = crate::pkg::compose::compose(consumer, &deps)?;
    std::fs::write(component_path, composed)?;
    Ok(())
}

/// Lib post-build: copy the package's exported WIT into `<output>/wit` so the
/// component + WIT form a self-contained, consumable artifact, then validate the
/// component's imports.
fn finalize_lib(manifest: &Manifest, component_path: &Path) -> Result<()> {
    let build = manifest.primary_build()?;
    let output = build.output.as_deref().unwrap_or("bin");
    let wit_src = build.wit.as_deref().ok_or_else(|| {
        eyre::eyre!("[lib.build].wit is required for `type = \"lib\"` (path to the package's WIT)")
    })?;

    let wit_dest = Path::new(output).join("wit");
    if wit_dest.exists() {
        std::fs::remove_dir_all(&wit_dest)?;
    }
    crate::fs::copy_dir_all(Path::new(wit_src), &wit_dest)?;

    validate_lib_component(component_path)?;

    println!("Package component: {}", component_path.display());
    println!("Package WIT:       {}", wit_dest.display());
    Ok(())
}

/// Ensures a `lib` component only imports `jumpjet:`/`wasi:` interfaces (any other
/// import would be unsatisfiable once composed into a game) and exports at least
/// one interface for consumers to use.
fn validate_lib_component(component_path: &Path) -> Result<()> {
    let bytes = std::fs::read(component_path)?;
    let decoded =
        wit_component::decode(&bytes).map_err(|e| eyre::eyre!("decoding component: {e}"))?;
    let (resolve, world_id) = match decoded {
        wit_component::DecodedWasm::Component(resolve, world) => (resolve, world),
        wit_component::DecodedWasm::WitPackage(..) => {
            return Err(eyre::eyre!("expected a component, found a WIT package"));
        }
    };
    let world = &resolve.worlds[world_id];

    let mut foreign = Vec::new();
    for key in world.imports.keys() {
        if let wit_parser::WorldKey::Interface(id) = key {
            if let Some(pkg_id) = resolve.interfaces[*id].package {
                let ns = &resolve.packages[pkg_id].name.namespace;
                if ns != "jumpjet" && ns != "wasi" {
                    foreign.push(resolve.id_of(*id).unwrap_or_else(|| ns.clone()));
                }
            }
        }
    }
    if !foreign.is_empty() {
        return Err(eyre::eyre!(
            "package imports interfaces outside `jumpjet:` and `wasi:` that cannot be satisfied: {}",
            foreign.join(", ")
        ));
    }

    let exports_iface = world
        .exports
        .keys()
        .any(|k| matches!(k, wit_parser::WorldKey::Interface(_)));
    if !exports_iface {
        return Err(eyre::eyre!(
            "package exports no interface; add `export <interface>` to its world"
        ));
    }
    Ok(())
}

/// Compiles the guest for the web: `pre` + componentize + `jco transpile` into
/// `<output>/web/guest`. Does NOT emit the host runtime / harness / page — that is
/// the assembly step shared by `bundle --target web` and `run --target web`
/// (see [`assemble_web_site`]).
pub async fn build_web(release: &bool) -> Result<()> {
    let manifest = Manifest::load()?;
    let primary = manifest.primary_build()?;
    let resolution = prepare_deps().await?;
    run_pre(primary)?;
    build_web_compile(primary, &resolution)?;
    println!("Web guest compiled to {}", web_guest_dir(primary)?.display());
    let _ = release;
    Ok(())
}

/// Like [`build_web`] but skips the `pre` command. Used by the `run --target web`
/// watch loop, which reacts to the developer's own compiler output rather than
/// invoking the guest compiler itself.
pub async fn build_web_incremental(_release: &bool) -> Result<()> {
    let manifest = Manifest::load()?;
    let primary = manifest.primary_build()?;
    let resolution = prepare_deps().await?;
    build_web_compile(primary, &resolution)?;
    Ok(())
}

/// Shared compile step: componentize the input, compose dependencies in, then
/// transpile the guest to JS.
fn build_web_compile(build: &Build, resolution: &Resolution) -> Result<()> {
    let component_path = componentize(build, ENTRYPOINT_FILE)?;
    compose_into(&component_path, resolution)?;
    let guest_dir = web_guest_dir(build)?;
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
            ));
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
    let wasm = parser
        .parse_file(&output_entrypoint_path)
        .expect("Unable to read game wasm");
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
            _ => {}
        }
    }

    if is_component {
        bytes = wasm;
    } else {
        // encoder = encoder.merge_imports_based_on_semver(merge); // TODO: Needed?
        encoder = encoder
            .module(&wasm)
            .expect("Unable to read game as a wasm module");

        let adapter = WasiWasm::get("wasi_snapshot_preview1.reactor.wasm").unwrap();
        let adapter = wat::parse_bytes(&adapter.data).unwrap();
        encoder = encoder
            .adapter("wasi_snapshot_preview1", &adapter)
            .expect("Unable to read adapter");

        bytes = encoder
            .encode()
            .expect("Failed to encode a component from provided module");
    }

    std::fs::write(&output_entrypoint_path, bytes).expect("Unable to write wasm");
}
