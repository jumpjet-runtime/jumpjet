pub mod android;
pub mod ios;
pub mod macos;
// pub mod windows;
pub mod web;

use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::OpenOptionsExt;

use current_platform::CURRENT_PLATFORM;
use semver::Version;
use toml::Table;
use xz2::read::XzDecoder;
use zip::ZipArchive;

use crate::Result;
use crate::settings::Settings;

pub async fn bundle(target: &String, release: &bool) -> Result<()> {
    let config = std::fs::read_to_string("jumpjet.toml")
        .unwrap()
        .parse::<Table>()
        .unwrap();
    let manifest = crate::pkg::manifest::Manifest::load()?;

    let current_dir = env::current_dir()?;
    let jumpjet_dir = current_dir.join(".jumpjet");
    let jumpjet_bin_dir = jumpjet_dir.join("bin");

    let build = if *release { "release" } else { "debug" }.to_owned();
    let target = target.to_owned();
    let target_triplet = match target.as_str() {
        "android" => "aarch64-linux-android",
        "ios" => "aarch64-apple-ios",
        "linux" => "x86_64-unknown-linux-musl",
        "macos" => "aarch64-apple-darwin",
        "windows" => "x86_64-pc-windows-msvc",
        "web" => "wasm32-wasip1",
        _ => panic!("no --target provided"),
    }
    .to_owned();

    let settings = Settings {
        current_dir: current_dir.clone(),
        jumpjet_dir,
        jumpjet_bin_dir,
        metadata_id: config["package"]["identifier"].as_str().unwrap().to_owned(),
        metadata_author: config["package"]["author"].as_str().unwrap().to_owned(),
        metadata_version: Version::parse(config["package"]["version"].as_str().unwrap()).unwrap(),
        build,
        target,
        target_triplet,
        runtime_version: Version::parse(config["runtime"]["version"].as_str().unwrap()).unwrap(),
        build_output_dir: current_dir
            .clone()
            .join(manifest.primary_build()?.output.as_deref().unwrap_or("bin")),
        bundle_name: config["bundle"]["name"].as_str().unwrap().to_owned(),
        bundle_identifier: config["package"]["identifier"].as_str().unwrap().to_owned(),

        ios_signing_identity: config
            .get("ios")
            .and_then(|t| t.get("signing-identity"))
            .and_then(|v| v.as_str())
            .map(str::to_owned),
        ios_provisioning_profile: config
            .get("ios")
            .and_then(|t| t.get("provisioning-profile"))
            .and_then(|v| v.as_str())
            .map(PathBuf::from),
        ios_min_os: config
            .get("ios")
            .and_then(|t| t.get("min-os"))
            .and_then(|v| v.as_str())
            .unwrap_or("13.0")
            .to_owned(),

        android_package: config
            .get("android")
            .and_then(|t| t.get("package"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| config["package"]["identifier"].as_str().unwrap())
            .to_owned(),
        android_min_sdk: config
            .get("android")
            .and_then(|t| t.get("min-sdk"))
            .and_then(|v| v.as_integer())
            .unwrap_or(24) as u32,
        android_target_sdk: config
            .get("android")
            .and_then(|t| t.get("target-sdk"))
            .and_then(|v| v.as_integer())
            .unwrap_or(34) as u32,
    };

    println!("Building for target {}", settings.target_triplet);

    // Componentize the guest into `bin/` (shared by every target).
    super::build::build(release).await?;

    // Scaffold the host wrapper (a `[[bin]]` for desktop/iOS, a `cdylib` exporting
    // `android_main` for Android) and the Rust toolchain used to cross-build it.
    init_rust_project(&settings).await?;
    install_rustup(&settings).await?;
    add_rust_target(&settings).await?;

    // Mobile targets diverge from the desktop zig pipeline: iOS needs the Apple
    // toolchain + an AOT precompile + an `.ipa`, Android needs the NDK + an
    // `.apk`. Each owns its build & packaging.
    match settings.target.as_str() {
        "ios" => return ios::bundle(&settings).await,
        "android" => return android::bundle(&settings).await,
        _ => {}
    }

    // Desktop/static targets: cross-compile the wrapper with cargo-zigbuild, then
    // package per platform.
    install_zig(&settings).await?;
    install_cargo_zigbuild(&settings).await?;
    build_target(&settings).await?;

    match settings.target.as_str() {
        "linux" => {}
        "macos" => macos::bundle_project(&settings)?,
        // "windows" => windows::bundle_project(&settings)?,
        // "web" => web::bundle_project(&settings)?,
        _ => {}
    }

    Ok(())
}

/// Scaffolds the host wrapper crate under `.jumpjet/project`. The shape depends on
/// the target: desktop/iOS produce a `[[bin]]`, Android a `cdylib` exporting
/// `android_main`. The sources are always rewritten (cheap) so switching `--target`
/// regenerates the correct wrapper rather than reusing a stale one.
async fn init_rust_project(settings: &Settings) -> Result<()> {
    let metadata_id = &settings.metadata_id;
    let runtime_version = settings.runtime_version.to_string();
    let entrypoint_path_str = crate::commands::build::ENTRYPOINT_FILE;

    let project_dir = settings.jumpjet_dir.join("project");
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    if settings.target == "android" {
        // cargo-apk builds a cdylib and packages it into an APK using
        // `[package.metadata.android]` for the manifest. The guest tree is shipped
        // as the single `assets/input.tar` extracted at startup by
        // `runtime::prepare_android_input` (Android can't enumerate asset subdirs).
        let cargotoml = format!(
            r#"[package]
name = "{metadata_id}"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
crate-type = ["cdylib"]

[dependencies]
jumpjet = {{ path = "../../../jumpjet/crates/jumpjet", version = "{runtime_version}" }}

[package.metadata.android]
package = "{package}"
build_targets = ["{triplet}"]
assets = "assets"

[package.metadata.android.sdk]
min_sdk_version = {min_sdk}
target_sdk_version = {target_sdk}

[[package.metadata.android.uses_feature]]
name = "android.hardware.vulkan.level"
required = false
"#,
            package = settings.android_package,
            triplet = settings.target_triplet,
            min_sdk = settings.android_min_sdk,
            target_sdk = settings.android_target_sdk,
        );
        fs::write(project_dir.join("Cargo.toml"), cargotoml)?;

        // `android_main` is the entry android-activity's NativeActivity glue calls.
        // We extract the bundled guest tree, read the componentized entrypoint, and
        // hand both to the runtime's Android event loop.
        let lib_rs = format!(
            r#"// Generated by `jumpjet bundle --target android`. Do not edit.
use jumpjet::winit::platform::android::activity::AndroidApp;

#[no_mangle]
fn android_main(app: AndroidApp) {{
    let input_path = jumpjet::runtime::prepare_android_input(&app);
    let binary = std::fs::read(input_path.join("{entrypoint_path_str}"))
        .expect("Failed to read the guest entrypoint from extracted assets");
    jumpjet::runtime::run_android(app, input_path, binary, false);
}}
"#
        );
        fs::write(src_dir.join("lib.rs"), lib_rs)?;
        return Ok(());
    }

    // Desktop + iOS: a plain binary that reads the entrypoint from the bundle's
    // `.jumpjet/input/` dir. On iOS that entrypoint file holds the Pulley `.cwasm`
    // (the runtime picks the AOT loader by target); elsewhere it's the wasm
    // component loaded via JIT.
    let cargotoml = format!(
        r#"[package]
name = "{metadata_id}"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
jumpjet = {{ path = "../../../jumpjet/crates/jumpjet", version = "{runtime_version}" }}

[[bin]]
name = "{metadata_id}"
path = "src/main.rs"
"#
    );
    fs::write(project_dir.join("Cargo.toml"), cargotoml)?;

    let main_rs = format!(
        r#"// Generated by `jumpjet bundle`. Do not edit.
use std::env;
use std::fs;
use jumpjet::runtime;

fn main() {{
    let input_path = env::current_exe().unwrap().parent().unwrap().join(".jumpjet/input/");
    let binary = fs::read(input_path.join("{entrypoint_path_str}")).expect("Failed to read the entrypoint");
    runtime::run(input_path, binary, false);
}}
"#
    );
    fs::write(src_dir.join("main.rs"), main_rs)?;

    Ok(())
}

async fn install_rustup(settings: &Settings) -> Result<()> {
    let rustup_dir = settings.jumpjet_bin_dir.join("rustup");
    if fs::metadata(&rustup_dir).is_ok() {
        return Ok(());
    }
    fs::create_dir_all(&rustup_dir)?;

    let os = env::consts::OS;
    let filename = match os {
        "windows" => "rustup-init.exe",
        _ => "rustup-init",
    };
    let url = format!(
        "https://static.rust-lang.org/rustup/dist/{}/{}",
        CURRENT_PLATFORM, filename
    );
    let resp = reqwest::get(url).await?;
    let rustup_content = resp.bytes().await?;

    let rustup_path = rustup_dir.join(filename);

    let mut open_options = OpenOptions::new();
    let open_options = open_options.write(true).create(true);

    #[cfg(not(target_os = "windows"))]
    let open_options = open_options.mode(0o755);

    let rustup_file = open_options.open(&rustup_path)?;

    let mut rustup_out = BufWriter::new(rustup_file);
    rustup_out.write_all(&rustup_content.as_ref())?;

    let mut cmd = Command::new(&rustup_path);

    cmd.env("CARGO_HOME", settings.jumpjet_bin_dir.join("cargo"))
        .env("RUSTUP_HOME", &rustup_dir)
        .arg("-y")
        .arg("-q")
        .arg("--no-update-default-toolchain")
        .arg("--no-modify-path");

    let output = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .output()?;

    let mut cmd = Command::new(rustup_path);

    cmd.env("CARGO_HOME", settings.jumpjet_bin_dir.join("cargo"))
        .env("RUSTUP_HOME", &rustup_dir)
        .arg("-y")
        .arg("-q")
        .arg("--no-update-default-toolchain")
        .arg("--no-modify-path");

    let output = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .output()?;

    // Configure Rust
    let rustup_path = settings.jumpjet_bin_dir.join("cargo/bin/rustup");
    let mut cmd = Command::new(&rustup_path);

    cmd.env("CARGO_HOME", settings.jumpjet_bin_dir.join("cargo"))
        .env("RUSTUP_HOME", &rustup_dir)
        .args(["default", "stable"]);

    let output = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .output()?;

    let mut cmd = Command::new(&rustup_path);

    cmd.env("CARGO_HOME", settings.jumpjet_bin_dir.join("cargo"))
        .env("RUSTUP_HOME", &rustup_dir)
        .args(["target", "add", "wasm32-wasip1"]);

    let output = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .output()?;

    Ok(())
}

async fn install_zig(settings: &Settings) -> Result<()> {
    let zig_dir = settings.jumpjet_bin_dir.join("zig");
    if zig_dir.exists() {
        return Ok(());
    }

    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    let version = "0.13.0";

    // Map Rust OS/Arch to Zig naming if necessary.
    // Rust: x86_64, aarch64
    // Zig: x86_64, aarch64 (mostly match)

    let (ext, filename) = if os == "windows" {
        ("zip", format!("zig-{}-{}-{}.zip", os, arch, version))
    } else {
        ("tar.xz", format!("zig-{}-{}-{}.tar.xz", os, arch, version))
    };

    let url = format!("https://ziglang.org/download/{}/{}", version, filename);
    println!("Downloading Zig from {}...", url);

    let resp = reqwest::get(url).await?;
    let content = resp.bytes().await?;

    let tarball_path = settings.jumpjet_bin_dir.join(&filename);
    let mut file = File::create(&tarball_path)?;
    file.write_all(&content)?;

    println!("Extracting Zig...");
    let file = File::open(&tarball_path)?;
    if ext == "zip" {
        let mut archive = ZipArchive::new(file)?;
        archive.extract(&settings.jumpjet_bin_dir)?;
    } else {
        let decoder = XzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(&settings.jumpjet_bin_dir)?;
    }

    // Rename extracted folder to "zig"
    // The folder name inside the archive is usually "zig-{os}-{arch}-{version}"
    let extracted_folder_name = filename.replace(&format!(".{}", ext), "");
    let extracted_path = settings.jumpjet_bin_dir.join(extracted_folder_name);

    if extracted_path.exists() {
        fs::rename(extracted_path, &zig_dir)?;
    } else {
        return Err(color_eyre::eyre::eyre!(
            "Failed to find extracted Zig folder"
        ));
    }

    // Clean up archive
    fs::remove_file(tarball_path)?;

    Ok(())
}

async fn install_cargo_zigbuild(settings: &Settings) -> Result<()> {
    // Check if cargo-zigbuild is installed
    let cargo_path = settings.jumpjet_bin_dir.join("cargo/bin/cargo");
    let zigbuild_check = Command::new(&cargo_path)
        .env("CARGO_HOME", settings.jumpjet_bin_dir.join("cargo"))
        .arg("zigbuild")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if let Ok(status) = zigbuild_check {
        if status.success() {
            return Ok(());
        }
    }

    println!("Installing cargo-zigbuild...");
    let mut cmd = Command::new(&cargo_path);
    cmd.env("CARGO_HOME", settings.jumpjet_bin_dir.join("cargo"));
    // The project's `.cargo/config.toml` pins `[build] target` to wasm for the
    // guest. cargo-zigbuild is a *host* tool, so force the host triple — otherwise
    // cargo tries to cross-compile it to wasm and pulls in code (e.g. `which`'s
    // wasi path) that needs the unstable `wasip2` feature and fails to build.
    cmd.env("CARGO_BUILD_TARGET", CURRENT_PLATFORM);
    // `--locked` installs with cargo-zigbuild's shipped Cargo.lock for reproducible
    // transitive deps.
    cmd.args(["install", "--locked", "cargo-zigbuild"]);

    let status = cmd.status()?;
    if !status.success() {
        return Err(color_eyre::eyre::eyre!("Failed to install cargo-zigbuild"));
    }

    Ok(())
}

async fn add_rust_target(settings: &Settings) -> Result<()> {
    let rustup_path = settings.jumpjet_bin_dir.join("cargo/bin/rustup");
    let mut cmd = Command::new(&rustup_path);

    cmd.env("CARGO_HOME", settings.jumpjet_bin_dir.join("cargo"))
        .env("RUSTUP_HOME", settings.jumpjet_bin_dir.join("rustup"))
        .args(["target", "add", &settings.target_triplet]);

    let output = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        return Err(color_eyre::eyre::eyre!(
            "Failed to add rust target: {}",
            settings.target_triplet
        ));
    }

    Ok(())
}

async fn build_target(settings: &Settings) -> Result<()> {
    let rust_project_path = settings.jumpjet_dir.join("project/.");

    let cargo_bin_path = settings.jumpjet_bin_dir.join("cargo/bin");

    let mut cmd = Command::new(cargo_bin_path.join("cargo"));
    cmd.current_dir(rust_project_path);

    // Add Zig to PATH
    let zig_bin = settings.jumpjet_bin_dir.join("zig");
    let path_env = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", zig_bin.to_str().unwrap(), path_env);

    cmd.env("CARGO_HOME", settings.jumpjet_bin_dir.join("cargo"))
        .env("RUSTUP_HOME", settings.jumpjet_bin_dir.join("rustup"))
        .env("PATH", new_path);

    cmd.args(["zigbuild", "--target", &settings.target_triplet]);
    // Match the profile to `settings.build` so the artifact lands where
    // `target_binary_path()` expects it (debug/ vs release/).
    if settings.build == "release" {
        cmd.arg("--release");
    }

    let output = cmd
        .stdout(Stdio::inherit()) // Changed to inherit to see build output
        .stderr(Stdio::inherit())
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}

async fn copy_output_to_input(settings: &Settings) -> Result<()> {
    crate::fs::copy_dir_all(
        &settings.build_output_dir,
        settings.jumpjet_dir.join("input"),
    )?;
    Ok(())
}
