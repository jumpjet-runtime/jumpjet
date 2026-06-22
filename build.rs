fn main() -> Result<(), Box<dyn std::error::Error>> {
    vergen::EmitBuilder::builder()
        .all_build()
        .all_git()
        .emit()?;

    for var in ["JUMPJET_FIREBASE_APP_ID", "JUMPJET_FIREBASE_API_SECRET"] {
        println!("cargo:rerun-if-env-changed={var}");
        let value = std::env::var(var).unwrap_or_default();
        println!("cargo:rustc-env={var}={value}");
    }

    Ok(())
}
