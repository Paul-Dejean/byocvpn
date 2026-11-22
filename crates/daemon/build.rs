#[cfg(windows)]
use std::{env::var, error::Error, fs::copy, path::Path};

fn main() {
    #[cfg(windows)]
    if let Err(error) = copy_wintun() {
        println!("cargo::error=failed to copy wintun.dll: {error}");
    }
}

#[cfg(windows)]
fn copy_wintun() -> Result<(), Box<dyn Error>> {
    let out_dir = var("OUT_DIR")?;
    let target_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .ok_or("OUT_DIR has fewer ancestors than expected")?
        .to_path_buf();

    let manifest_dir = var("CARGO_MANIFEST_DIR")?;
    let wintun_src = Path::new(&manifest_dir).join("../../vendor/wintun/wintun.dll");

    if wintun_src.exists() {
        let dst = target_dir.join("wintun.dll");
        copy(&wintun_src, &dst)?;
        println!("cargo:warning=Copied wintun.dll to {}", dst.display());
    } else {
        println!(
            "cargo:warning=wintun.dll not found at {}, TUN device will fail at runtime",
            wintun_src.display()
        );
    }

    println!("cargo:rerun-if-changed=../../vendor/wintun/wintun.dll");
    Ok(())
}
