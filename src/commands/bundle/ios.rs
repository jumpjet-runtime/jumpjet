//! iOS bundling: AOT-precompile the guest to Pulley, cross-build the wrapper with
//! the Apple toolchain, assemble a `Payload/<Name>.app`, codesign it, and zip an
//! `.ipa`.
//!
//! Unlike the desktop targets this can't use cargo-zigbuild — iOS linking needs
//! Xcode's `clang`/SDK (`xcrun`). Device installs additionally need a signing
//! identity + provisioning profile (configure `[ios]` in `jumpjet.toml`); without
//! one the `.app` is left unsigned, which only the Simulator accepts.

use std::path::Path;
use std::process::{Command, Stdio};

use color_eyre::eyre::eyre;

use crate::Result;
use crate::settings::Settings;

pub async fn bundle(settings: &Settings) -> Result<()> {
    ensure_xcode()?;

    // 1. AOT: precompile the componentized guest to a Pulley `.cwasm`. iOS forbids
    //    JIT, so the runtime loads this interpreted via `Game::from_cwasm`.
    let entrypoint = settings
        .build_output_dir
        .join(crate::commands::build::ENTRYPOINT_FILE);
    let wasm = std::fs::read(&entrypoint)
        .map_err(|e| eyre!("reading componentized guest {}: {e}", entrypoint.display()))?;
    println!("Precompiling guest to Pulley bytecode (iOS AOT)...");
    let cwasm = jumpjet::aot::precompile_pulley(&wasm)
        .map_err(|e| eyre!("Pulley precompile failed: {e}"))?;

    // 2. Build the wrapper binary for the device triple with the Apple toolchain.
    build_apple(settings)?;

    // 3. Assemble Payload/<Name>.app.
    let bundle_root = settings.current_dir.join("bundle/ios");
    let payload = bundle_root.join("Payload");
    let app = payload.join(format!("{}.app", settings.bundle_name));
    if app.exists() {
        std::fs::remove_dir_all(&app)?;
    }
    std::fs::create_dir_all(&app)?;

    // Guest tree (assets + the entrypoint, which we replace with the `.cwasm`).
    let input_dest = app.join(".jumpjet/input");
    crate::fs::copy_dir_all(&settings.build_output_dir, &input_dest)?;
    std::fs::write(
        input_dest.join(crate::commands::build::ENTRYPOINT_FILE),
        &cwasm,
    )?;

    // Executable.
    crate::fs::copy_file(
        settings.target_binary_path(),
        app.join(settings.binary_name()),
    )?;

    write_info_plist(&app, settings)?;

    if let Some(profile) = &settings.ios_provisioning_profile {
        crate::fs::copy_file(profile.clone(), app.join("embedded.mobileprovision"))?;
    }

    // 4. Codesign (device only; Simulator runs unsigned).
    codesign(&app, settings)?;

    // 5. Zip Payload/ into <Name>.ipa.
    let ipa = bundle_root.join(format!("{}.ipa", settings.bundle_name));
    if ipa.exists() {
        std::fs::remove_file(&ipa)?;
    }
    zip_ipa(&bundle_root, &ipa)?;

    println!("Built {}", ipa.display());
    Ok(())
}

/// Fails fast with actionable guidance if the Apple toolchain is missing — iOS
/// builds are impossible without Xcode's SDK and linker.
fn ensure_xcode() -> Result<()> {
    let ok = Command::new("xcrun")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        Ok(())
    } else {
        Err(eyre!(
            "iOS bundling requires Xcode (the `xcrun` toolchain). Install Xcode and run \
             `xcode-select --install`, then retry."
        ))
    }
}

/// Cross-builds the wrapper for `settings.target_triplet` (e.g. aarch64-apple-ios)
/// with the bootstrapped Rust toolchain. rustc drives the Apple linker via `cc`,
/// which resolves to Xcode's clang, so `PATH` must keep the system entries.
fn build_apple(settings: &Settings) -> Result<()> {
    let cargo = settings.jumpjet_bin_dir.join("cargo/bin/cargo");
    let mut cmd = Command::new(cargo);
    cmd.current_dir(settings.project_dir())
        .env("CARGO_HOME", settings.jumpjet_bin_dir.join("cargo"))
        .env("RUSTUP_HOME", settings.jumpjet_bin_dir.join("rustup"))
        .args(["build", "--target", &settings.target_triplet]);
    // Match the profile to `settings.build` so the artifact lands where
    // `target_binary_path()` expects it (debug/ vs release/).
    if settings.build == "release" {
        cmd.arg("--release");
    }
    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        return Err(eyre!("cargo build for {} failed", settings.target_triplet));
    }
    Ok(())
}

/// `codesign` the app with the configured identity. Skipped (with a warning) when
/// no identity is set — the resulting `.ipa` then only runs on the Simulator.
fn codesign(app: &Path, settings: &Settings) -> Result<()> {
    let Some(identity) = &settings.ios_signing_identity else {
        eprintln!(
            "warning: no [ios] signing-identity configured — leaving the app unsigned \
             (Simulator-only; device installs will be rejected)."
        );
        return Ok(());
    };

    let mut cmd = Command::new("codesign");
    cmd.arg("--force")
        .arg("--sign")
        .arg(identity)
        .arg("--timestamp=none");
    if let Some(profile) = &settings.ios_provisioning_profile {
        // Entitlements are normally extracted from the profile; pass it through so
        // codesign can embed the matching entitlements.
        cmd.arg("--entitlements").arg(profile);
    }
    let status = cmd
        .arg(app)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        return Err(eyre!("codesign failed for {}", app.display()));
    }
    Ok(())
}

/// Zips `<bundle_root>/Payload` into the `.ipa` (an `.ipa` is just a zip whose
/// root is `Payload/`). Uses the system `zip`, which is always present on macOS
/// where iOS builds run.
fn zip_ipa(bundle_root: &Path, ipa: &Path) -> Result<()> {
    let status = Command::new("zip")
        .current_dir(bundle_root)
        .arg("-r")
        .arg("-q")
        .arg(ipa)
        .arg("Payload")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        return Err(eyre!("zipping {} failed", ipa.display()));
    }
    Ok(())
}

fn write_info_plist(app: &Path, settings: &Settings) -> Result<()> {
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>{name}</string>
  <key>CFBundleExecutable</key>
  <string>{exe}</string>
  <key>CFBundleIdentifier</key>
  <string>{id}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>{name}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>{version}</string>
  <key>CFBundleVersion</key>
  <string>{version}</string>
  <key>LSRequiresIPhoneOS</key>
  <true/>
  <key>MinimumOSVersion</key>
  <string>{min_os}</string>
  <key>CFBundleSupportedPlatforms</key>
  <array>
    <string>iPhoneOS</string>
  </array>
  <key>UIDeviceFamily</key>
  <array>
    <integer>1</integer>
    <integer>2</integer>
  </array>
  <key>UIRequiredDeviceCapabilities</key>
  <array>
    <string>arm64</string>
  </array>
  <key>UILaunchScreen</key>
  <dict/>
</dict>
</plist>
"#,
        name = settings.bundle_name,
        exe = settings.binary_name(),
        id = settings.bundle_identifier,
        version = settings.metadata_version,
        min_os = settings.ios_min_os,
    );
    std::fs::write(app.join("Info.plist"), plist)?;
    Ok(())
}
