use std::{
    env, fs::File, io::{Cursor, Read}, path::PathBuf
};

use flate2::bufread::GzDecoder;
use reqwest::{header::{HeaderMap, HeaderValue, USER_AGENT}, Client};
use tar::Archive;
use serde_json::Value;
use tempfile::TempDir;

use crate::Result;

pub async fn upgrade() -> Result<()> {
    let latest_version = {
        let client = Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Jumpjet CLI (https://jumpjet.dev)"));

        let response = client
            .get("https://api.github.com/repos/jumpjet-runtime/jumpjet/tags")
            .headers(headers)
            .send()
            .await?;

        let json = response.text().await?;
        let value: Value = serde_json::from_str(&json)?;
        value[0]["name"].as_str().unwrap().to_string()
    };
    let platform = format!("{}-{}", env::consts::OS, env::consts::ARCH);
    let tarball_name = format!("jumpjet-cli-{latest_version}-{platform}.tar.gz");
    let tarball_url = format!("https://github.com/jumpjet-runtime/jumpjet/releases/download/{latest_version}/{tarball_name}");

    let tmp_dir = tempfile::Builder::new()
        .prefix(&format!(".update-{latest_version}"))
        .tempdir_in(::std::env::current_dir()?)?;

    let tmp_path = tmp_dir.path().to_path_buf();

    download_tarball(
        &tarball_url,
        &tmp_path
    ).await?;

    let new_bin = tmp_path.join("jumpjet-cli");

    self_replace::self_replace(new_bin)?;

    println!("Successfully upgraded to {latest_version}");

    Ok(())
}

async fn download_tarball(url: &str, tmp_path: &PathBuf) -> Result<()> {
    let response = reqwest::get(url).await?;
    let tmp_tarball = tmp_path.join("jumpjet.tar.gz");
    let targz_bytes = response.bytes().await?;

    let mut gz = GzDecoder::new(&targz_bytes[..]);
    let mut tar_bytes = Vec::<u8>::new();
    gz.read_to_end(&mut tar_bytes)?;
    let mut archive = Archive::new(&tar_bytes[..]);
    archive.unpack(tmp_path)?;

    Ok(())
}
