fn main() {
    #[cfg(windows)]
    {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        // OUT_DIR is like target/debug/build/<pkg>/out — walk up to target/<profile>
        let target_dir = std::path::Path::new(&out_dir)
            .ancestors()
            .nth(3)
            .unwrap()
            .to_path_buf();

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let wintun_src = std::path::Path::new(&manifest_dir)
            .join("../../vendor/wintun/wintun.dll");

        if wintun_src.exists() {
            let dst = target_dir.join("wintun.dll");
            std::fs::copy(&wintun_src, &dst).expect("Failed to copy wintun.dll");
            println!("cargo:warning=Copied wintun.dll to {}", dst.display());
        } else {
            println!("cargo:warning=wintun.dll not found at {}, TUN device will fail at runtime", wintun_src.display());
        }

        println!("cargo:rerun-if-changed=../../vendor/wintun/wintun.dll");
    }
}
