use std::path::PathBuf;

use semver::Version;

pub struct Settings {
    pub current_dir: PathBuf,
    pub jumpjet_dir: PathBuf,
    pub jumpjet_bin_dir: PathBuf,

    pub metadata_id: String,
    pub metadata_version: Version,
    pub metadata_author: String,

    pub build: String,
    pub target: String,
    pub target_triplet: String,

    pub runtime_version: Version,

    pub build_output_dir: PathBuf,

    pub bundle_name: String,
    pub bundle_identifier: String,

    /// Absolute path to the source icon image (e.g. a PNG) declared as
    /// `[bundle].icon` in `jumpjet.toml`. Each platform bundler converts it to its
    /// native format (`.icns` on macOS, `.ico` on Windows). `None` ships no icon.
    pub icon: Option<PathBuf>,

    /// iOS codesign identity (e.g. "Apple Development: you@example.com"). `None`
    /// leaves the app unsigned — fine for the Simulator, rejected on device.
    pub ios_signing_identity: Option<String>,
    /// Path to the `.mobileprovision` embedded into the `.app` for device builds.
    pub ios_provisioning_profile: Option<PathBuf>,
    /// `MinimumOSVersion` written into the iOS `Info.plist`.
    pub ios_min_os: String,

    /// Android application id (reverse-DNS); defaults to the package identifier.
    pub android_package: String,
    /// Android `minSdkVersion` / `targetSdkVersion`.
    pub android_min_sdk: u32,
    pub android_target_sdk: u32,
}

impl Settings {
    pub fn binary_name(&self) -> String {
        "".to_owned()
            + &self.metadata_id
            + match self.target.as_str() {
                "windows" => ".exe",
                _ => "",
            }
    }

    pub fn project_dir(&self) -> PathBuf {
        self.jumpjet_dir.join("project/.")
    }

    pub fn target_dir(&self) -> PathBuf {
        self.project_dir()
            .join("target")
            .join(&self.target_triplet)
            .join(&self.build)
    }

    pub fn target_binary_path(&self) -> PathBuf {
        self.target_dir().join(self.binary_name())
    }
}
