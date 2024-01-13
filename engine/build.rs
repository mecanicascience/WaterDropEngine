#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]

use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() -> Result<()> {
    // This tells cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=res/*");

    // Copy the /res/ folder to the output directory.
    let mut copyOptions = CopyOptions::new();
    copyOptions.overwrite = true;
    let mut pathsToCopy = Vec::new();
    pathsToCopy.push("res/");
    copy_items(&pathsToCopy, "./target/debug/", &copyOptions)?;
    copy_items(&pathsToCopy, "./target/release/", &copyOptions)?;

    Ok(())
}