use crate::Result;

/// Re-resolves the project's `[dependencies]` from scratch and rewrites
/// `jumpjet.lock` (cargo-`update` style). Regular `build` honors the existing
/// lock; `update` refreshes it.
pub async fn update() -> Result<()> {
    let dir = std::env::current_dir()?;
    let resolution = crate::pkg::resolve::resolve(&dir, true).await?;

    if resolution.packages.is_empty() {
        println!("No dependencies to update.");
        return Ok(());
    }

    println!("Updated {} package(s):", resolution.packages.len());
    for pkg in &resolution.packages {
        println!("  {} -> {}", pkg.id, pkg.stored.dir.display());
    }
    Ok(())
}
