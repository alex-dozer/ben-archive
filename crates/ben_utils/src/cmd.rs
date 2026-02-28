use anyhow::{Context, Result, anyhow};
use serde_json::from_str;
use std::process::{Command, Stdio};

use crate::schema::manifest::SchemaManifest;

pub fn run_build_script(cmdline: &str) -> Result<String> {
    let output = Command::new("/bin/sh")
        .arg("-c")
        .arg(cmdline)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .context("executing build script")?;

    if !output.status.success() {
        return Err(anyhow!("build script failed with status {}", output.status));
    }
    let stdout = String::from_utf8(output.stdout).context("build script stdout not utf-8")?;
    Ok(stdout)
}

pub fn parse_manifest(json: &str) -> Result<SchemaManifest> {
    Ok(from_str::<SchemaManifest>(json).context("manifest json parse error")?)
}
