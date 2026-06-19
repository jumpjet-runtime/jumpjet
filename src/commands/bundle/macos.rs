use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

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

/// Given a list of icon files, try to produce an ICNS file in the resources
/// directory and return the path to it.  Returns `Ok(None)` if no usable icons
/// were provided.
fn create_icns_file(
    resources_dir: &PathBuf,
    settings: &Settings,
) -> std::io::Result<Option<PathBuf>> {
    // if settings.icon_files().count() == 0 {
    //     return Ok(None);
    // }

    // // If one of the icon files is already an ICNS file, just use that.
    // for icon_path in settings.icon_files() {
    //     let icon_path = icon_path?;
    //     if icon_path.extension() == Some(OsStr::new("icns")) {
    //         let mut dest_path = resources_dir.to_path_buf();
    //         dest_path.push(icon_path.file_name().unwrap());
    //         crate::fs::copy_file(&icon_path, &dest_path)?;
    //         return Ok(Some(dest_path));
    //     }
    // }

    // Otherwise, read available images and pack them into a new ICNS file.
    let family = icns::IconFamily::new();

    fn add_icon_to_family(
        icon: image::DynamicImage,
        density: u32,
        family: &mut icns::IconFamily,
    ) -> std::io::Result<()> {
        // Try to add this image to the icon family.  Ignore images whose sizes
        // don't map to any ICNS icon type; print warnings and skip images that
        // fail to encode.
        match icns::IconType::from_pixel_size_and_density(icon.width(), icon.height(), density) {
            Some(icon_type) => {
                if !family.has_icon_with_type(icon_type) {
                    let icon = make_icns_image(icon)?;
                    family.add_icon_with_type(&icon, icon_type)?;
                }
                Ok(())
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No matching IconType",
            )),
        }
    }

    // let mut images_to_resize: Vec<(image::DynamicImage, u32, u32)> = vec![];
    // for icon_path in settings.icon_files() {
    //     let icon_path = icon_path?;
    //     let icon = image::open(&icon_path)?;
    //     let density = if common::is_retina(&icon_path) { 2 } else { 1 };
    //     let (w, h) = icon.dimensions();
    //     let orig_size = min(w, h);
    //     let next_size_down = 2f32.powf((orig_size as f32).log2().floor()) as u32;
    //     if orig_size > next_size_down {
    //         images_to_resize.push((icon, next_size_down, density));
    //     } else {
    //         add_icon_to_family(icon, density, &mut family)?;
    //     }
    // }

    // for (icon, next_size_down, density) in images_to_resize {
    //     let icon = icon.resize_exact(next_size_down, next_size_down, image::Lanczos3);
    //     add_icon_to_family(icon, density, &mut family)?;
    // }

    // if !family.is_empty() {
    //     fs::create_dir_all(resources_dir)?;
    //     let mut dest_path = resources_dir.clone();
    //     dest_path.push(settings.bundle_name());
    //     dest_path.set_extension("icns");
    //     let icns_file = BufWriter::new(File::create(&dest_path)?);
    //     family.write(icns_file)?;
    //     return Ok(Some(dest_path));
    // }

    Ok(None)

    // bail!("No usable icon files found.");
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
