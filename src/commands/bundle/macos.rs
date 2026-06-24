use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use color_eyre::eyre::eyre;
use image::GenericImageView;

use crate::settings::Settings;

pub fn bundle_project(settings: &Settings) -> crate::Result<()> {
    let app_bundle_name = format!("{}.app", settings.bundle_name);
    let app_bundle_path = settings
        .current_dir
        .join("bundle/macos")
        .join(&app_bundle_name);
    if app_bundle_path.exists() {
        fs::remove_dir_all(&app_bundle_path)?;
    }

    let bundle_directory = app_bundle_path.join("Contents");
    fs::create_dir_all(&bundle_directory)?;

    let resources_dir = bundle_directory.join("Resources");

    let bundle_icon_file: Option<PathBuf> = { create_icns_file(&resources_dir, settings)? };

    create_info_plist(&bundle_directory, bundle_icon_file, settings)?;

    copy_frameworks_to_bundle(&bundle_directory, settings)?;

    copy_build_output_to_bundle(&bundle_directory, settings)?;

    Ok(())
}

fn copy_build_output_to_bundle(bundle_directory: &Path, settings: &Settings) -> crate::Result<()> {
    let dest_dir = bundle_directory.join("MacOS");
    crate::fs::copy_dir_all(&settings.build_output_dir, dest_dir.join(".jumpjet/input"))?;
    crate::fs::copy_file(
        settings.target_binary_path(),
        dest_dir.join(settings.binary_name()),
    )?;
    Ok(())
}

fn create_info_plist(
    bundle_dir: &Path,
    bundle_icon_file: Option<PathBuf>,
    settings: &Settings,
) -> crate::Result<()> {
    let build_number = chrono::Utc::now().format("%Y%m%d.%H%M%S");
    let file = &mut crate::fs::create_file(&bundle_dir.join("Info.plist"))?;
    write!(
        file,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
            <!DOCTYPE plist PUBLIC \"-//Apple Computer//DTD PLIST 1.0//EN\" \
            \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
            <plist version=\"1.0\">\n\
            <dict>\n"
    )?;
    write!(
        file,
        "  <key>CFBundleDevelopmentRegion</key>\n  \
            <string>English</string>\n"
    )?;
    write!(
        file,
        "  <key>CFBundleDisplayName</key>\n  <string>{}</string>\n",
        settings.bundle_name
    )?;
    write!(
        file,
        "  <key>CFBundleExecutable</key>\n  <string>{}</string>\n",
        settings.binary_name()
    )?;
    if let Some(path) = bundle_icon_file {
        write!(
            file,
            "  <key>CFBundleIconFile</key>\n  <string>{}</string>\n",
            path.file_name().unwrap().to_string_lossy()
        )?;
    }
    write!(
        file,
        "  <key>CFBundleIdentifier</key>\n  <string>{}</string>\n",
        settings.bundle_identifier
    )?;
    write!(
        file,
        "  <key>CFBundleInfoDictionaryVersion</key>\n  \
            <string>6.0</string>\n"
    )?;
    write!(
        file,
        "  <key>CFBundleName</key>\n  <string>{}</string>\n",
        settings.bundle_name
    )?;
    write!(
        file,
        "  <key>CFBundlePackageType</key>\n  <string>APPL</string>\n"
    )?;
    write!(
        file,
        "  <key>CFBundleShortVersionString</key>\n  <string>{}</string>\n",
        settings.metadata_version.to_string()
    )?;
    // if !settings.osx_url_schemes().is_empty() {
    //     write!(
    //         file,
    //         "  <key>CFBundleURLTypes</key>\n  \
    //            <array>\n    \
    //                <dict>\n      \
    //                    <key>CFBundleURLName</key>\n      \
    //                    <string>{}</string>\n      \
    //                    <key>CFBundleTypeRole</key>\n      \
    //                    <string>Viewer</string>\n      \
    //                    <key>CFBundleURLSchemes</key>\n      \
    //                    <array>\n",
    //         settings.bundle_name()
    //     )?;
    //     for scheme in settings.osx_url_schemes() {
    //         writeln!(file, "        <string>{scheme}</string>")?;
    //     }
    //     write!(
    //         file,
    //         "      </array>\n    \
    //             </dict>\n  \
    //          </array>\n"
    //     )?;
    // }
    write!(
        file,
        "  <key>CFBundleVersion</key>\n  <string>{build_number}</string>\n"
    )?;
    write!(file, "  <key>CSResourcesFileMapped</key>\n  <true/>\n")?;
    // if let Some(category) = settings.app_category() {
    //     write!(
    //         file,
    //         "  <key>LSApplicationCategoryType</key>\n  \
    //             <string>{}</string>\n",
    //         category.osx_application_category_type()
    //     )?;
    // }
    // if let Some(version) = settings.osx_minimum_system_version() {
    //     write!(
    //         file,
    //         "  <key>LSMinimumSystemVersion</key>\n  \
    //             <string>{version}</string>\n"
    //     )?;
    // }
    write!(file, "  <key>LSRequiresCarbon</key>\n  <true/>\n")?;
    write!(file, "  <key>NSHighResolutionCapable</key>\n  <true/>\n")?;
    // if let Some(copyright) = settings.copyright_string() {
    //     write!(
    //         file,
    //         "  <key>NSHumanReadableCopyright</key>\n  \
    //             <string>{copyright}</string>\n"
    //     )?;
    // }
    write!(file, "</dict>\n</plist>\n")?;
    file.flush()?;
    Ok(())
}

fn copy_framework_from(dest_dir: &Path, framework: &str, src_dir: &Path) -> crate::Result<bool> {
    let src_name = format!("{framework}.framework");
    let src_path = src_dir.join(&src_name);
    if src_path.exists() {
        crate::fs::copy_dir_all(&src_path, &dest_dir.join(&src_name))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn copy_frameworks_to_bundle(bundle_directory: &Path, settings: &Settings) -> crate::Result<()> {
    // let frameworks = settings.osx_frameworks();
    // if frameworks.is_empty() {
    //     return Ok(());
    // }
    // let dest_dir = bundle_directory.join("Frameworks");
    // fs::create_dir_all(bundle_directory)?;
    // for framework in frameworks.iter() {
    //     if framework.ends_with(".framework") {
    //         let src_path = PathBuf::from(framework);
    //         let src_name = src_path.file_name().unwrap();
    //         crate::fs::copy_dir_all(&src_path, &dest_dir.join(src_name))?;
    //         continue;
    //     } else if framework.contains('/') {
    //         bail!(
    //             "Framework path should have .framework extension: {}",
    //             framework
    //         );
    //     }
    //     if let Some(home_dir) = dirs::home_dir() {
    //         if copy_framework_from(&dest_dir, framework, &home_dir.join("Library/Frameworks/"))? {
    //             continue;
    //         }
    //     }
    //     if copy_framework_from(&dest_dir, framework, &PathBuf::from("/Library/Frameworks/"))?
    //         || copy_framework_from(
    //             &dest_dir,
    //             framework,
    //             &PathBuf::from("/Network/Library/Frameworks/"),
    //         )?
    //         || copy_framework_from(
    //             &dest_dir,
    //             framework,
    //             &PathBuf::from("/System/Library/Frameworks/"),
    //         )?
    //     {
    //         continue;
    //     }
    //     bail!("Could not locate {}.framework", framework);
    // }
    Ok(())
}

/// Produces an `.icns` in the resources directory from the configured
/// `[bundle].icon` source image, returning the path to it. Returns `Ok(None)` when
/// no icon is configured. If the source is already an `.icns`, it's copied as-is;
/// otherwise it's resized into every ICNS slot the source resolution can fill.
fn create_icns_file(
    resources_dir: &Path,
    settings: &Settings,
) -> crate::Result<Option<PathBuf>> {
    let Some(icon_path) = settings.icon.as_ref() else {
        return Ok(None);
    };

    let dest_path = resources_dir.join(format!("{}.icns", settings.bundle_name));

    // If the source is already an ICNS file, just copy it over.
    if icon_path.extension() == Some(OsStr::new("icns")) {
        fs::create_dir_all(resources_dir)?;
        crate::fs::copy_file(icon_path.clone(), dest_path.clone())?;
        return Ok(Some(dest_path));
    }

    // Otherwise read the source image and pack it into a new ICNS file, resizing
    // into each standard ICNS size/density slot. We never upscale past the
    // source's resolution, so a small icon simply fills fewer slots.
    let source = image::open(icon_path)
        .map_err(|e| eyre!("reading [bundle].icon {}: {e}", icon_path.display()))?;
    let source_size = source.width().min(source.height());

    let mut family = icns::IconFamily::new();
    for &size in &[16u32, 32, 64, 128, 256, 512, 1024] {
        if size > source_size {
            continue;
        }
        for density in [1u32, 2] {
            let Some(icon_type) = icns::IconType::from_pixel_size_and_density(size, size, density)
            else {
                continue;
            };
            if family.has_icon_with_type(icon_type) {
                continue;
            }
            let resized = source.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
            family.add_icon_with_type(&make_icns_image(resized)?, icon_type)?;
        }
    }

    if family.is_empty() {
        return Ok(None);
    }

    fs::create_dir_all(resources_dir)?;
    let icns_file = BufWriter::new(File::create(&dest_path)?);
    family.write(icns_file)?;
    Ok(Some(dest_path))
}

/// Converts an image::DynamicImage into an icns::Image.
fn make_icns_image(img: image::DynamicImage) -> std::io::Result<icns::Image> {
    let pixel_format = match img.color() {
        image::ColorType::Rgba8 => icns::PixelFormat::RGBA,
        image::ColorType::Rgb8 => icns::PixelFormat::RGB,
        image::ColorType::La8 => icns::PixelFormat::GrayAlpha,
        image::ColorType::L8 => icns::PixelFormat::Gray,
        _ => {
            let msg = format!("unsupported ColorType: {:?}", img.color());
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, msg));
        }
    };
    icns::Image::from_data(
        pixel_format,
        img.width(),
        img.height(),
        img.pixels()
            .flat_map(|(_, _, p)| p.0)
            .collect::<Vec<_>>()
            .to_vec(),
    )
}
