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

    pub build_input_dir: PathBuf,
    pub build_output_dir: PathBuf,
    pub build_entrypoint: PathBuf,

    pub bundle_name: String,
    pub bundle_identifier: String,
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
