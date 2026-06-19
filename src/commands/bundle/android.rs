//! Android bundling: pack the guest tree into the wrapper's assets, then build,
//! package, and sign an `.apk` with cargo-apk (which drives the NDK + Android SDK
//! build tools).
//!
//! Like iOS-needs-Xcode, this needs the Android toolchain present: `ANDROID_HOME`
//! (a.k.a. `ANDROID_SDK_ROOT`) with build-tools, and an NDK. We don't auto-install
//! the SDK (the license/`sdkmanager` dance is fragile); instead we detect it and
//! fail with guidance. cargo-apk debug-signs the APK so `adb install` works out of
//! the box; release-keystore signing is left to project config.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use color_eyre::eyre::eyre;

use crate::Result;
use crate::settings::Settings;

pub async fn bundle(settings: &Settings) -> Result<()> {
    let (sdk, ndk) = ensure_sdk()?;
    install_cargo_apk(settings)?;

    // Pack the guest tree (componentized entrypoint + assets) into the single
    // `assets/input.tar` the runtime extracts at startup — Android can't enumerate
    // asset subdirectories, so one archive is the reliable shipping unit.
    write_input_archive(settings)?;

    build_apk(settings, &sdk, &ndk)?;

    // Collect the APK cargo-apk produced under the project target dir.
    let apk = find_apk(&settings.project_dir())?;
    let dest_dir = settings.current_dir.join("bundle/android");
    std::fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join(format!("{}.apk", settings.bundle_name));
    crate::fs::copy_file(apk, dest.clone())?;

    println!("Built {}", dest.display());
    Ok(())
}

/// Resolves the Android SDK and NDK locations from the environment, erroring with
/// guidance if either is missing.
fn ensure_sdk() -> Result<(PathBuf, PathBuf)> {
    let sdk = std::env::var_os("ANDROID_HOME")
        .or_else(|| std::env::var_os("ANDROID_SDK_ROOT"))
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .ok_or_else(|| {
            eyre!(
                "Android bundling needs the Android SDK. Install it (e.g. via Android \
                 Studio) and set ANDROID_HOME to the SDK path."
            )
        })?;

    // Prefer an explicit NDK env; otherwise pick the newest under `<sdk>/ndk`.
    let ndk = std::env::var_os("ANDROID_NDK_ROOT")
        .or_else(|| std::env::var_os("ANDROID_NDK_HOME"))
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .or_else(|| newest_ndk(&sdk.join("ndk")))
        .ok_or_else(|| {
            eyre!(
                "No Android NDK found. Install one (`sdkmanager 'ndk;<version>'`) and set \
                 ANDROID_NDK_ROOT, or place it under $ANDROID_HOME/ndk."
            )
        })?;

    Ok((sdk, ndk))
}

/// Newest versioned NDK directory under `<sdk>/ndk`, if any.
fn newest_ndk(ndk_root: &Path) -> Option<PathBuf> {
    let mut versions: Vec<PathBuf> = std::fs::read_dir(ndk_root)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    versions.sort();
    versions.pop()
}

/// Installs cargo-apk into the bootstrapped cargo home if it isn't already there.
fn install_cargo_apk(settings: &Settings) -> Result<()> {
    let cargo = settings.jumpjet_bin_dir.join("cargo/bin/cargo");
    let cargo_home = settings.jumpjet_bin_dir.join("cargo");

    let present = Command::new(&cargo)
        .env("CARGO_HOME", &cargo_home)
        .args(["apk", "--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if present {
        return Ok(());
    }

    println!("Installing cargo-apk...");
    let status = Command::new(&cargo)
        .env("CARGO_HOME", &cargo_home)
        .args(["install", "cargo-apk"])
        .status()?;
    if !status.success() {
        return Err(eyre!("Failed to install cargo-apk"));
    }
    Ok(())
}

/// Tars the build output dir into `<project>/assets/input.tar`. Entries are
/// relative to the output root, so extraction restores `<entrypoint>` and any
/// asset files directly under the runtime's input path.
fn write_input_archive(settings: &Settings) -> Result<()> {
    let assets_dir = settings.project_dir().join("assets");
    std::fs::create_dir_all(&assets_dir)?;
    let archive = assets_dir.join("input.tar");

    let file = std::fs::File::create(&archive)?;
    let mut builder = tar::Builder::new(file);
    builder.append_dir_all(".", &settings.build_output_dir)?;
    builder.finish()?;
    Ok(())
}

/// Builds + packages + signs the APK with cargo-apk, pointing it at the resolved
/// SDK/NDK. cargo-apk reads `[package.metadata.android]` from the wrapper crate.
fn build_apk(settings: &Settings, sdk: &Path, ndk: &Path) -> Result<()> {
    let cargo = settings.jumpjet_bin_dir.join("cargo/bin/cargo");
    let status = Command::new(cargo)
        .current_dir(settings.project_dir())
        .env("CARGO_HOME", settings.jumpjet_bin_dir.join("cargo"))
        .env("RUSTUP_HOME", settings.jumpjet_bin_dir.join("rustup"))
        .env("ANDROID_HOME", sdk)
        .env("ANDROID_SDK_ROOT", sdk)
        .env("ANDROID_NDK_ROOT", ndk)
        .args(["apk", "build", "--release"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        return Err(eyre!("cargo apk build failed"));
    }
    Ok(())
}

/// Finds the APK cargo-apk emitted under `<project>/target/**/apk/*.apk`,
/// returning the most recently modified one.
fn find_apk(project_dir: &Path) -> Result<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "apk") {
                out.push(path);
            }
        }
    }

    let mut apks = Vec::new();
    walk(&project_dir.join("target"), &mut apks);
    apks.into_iter()
        .max_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok())
        .ok_or_else(|| eyre!("cargo apk produced no .apk under the target directory"))
}
