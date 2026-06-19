use rust_embed::Embed;

#[derive(Embed)]
#[folder = "crates/jumpjet/wit/runtime"]
pub struct JumpjetRuntimeWits;

#[derive(Embed)]
#[folder = "src/templates"]
pub struct Templates;
