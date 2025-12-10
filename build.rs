use std::{env, fs, path::Path};

fn main() {
    println!("cargo:warning=Running build.rs to copy config.json");

    // OUT_DIR = target/debug/build/<crate>/out
    let out_dir = env::var("OUT_DIR").expect("Cannot read OUT_DIR");

    // Move up 3 directories to reach target/debug or target/release
    let exe_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .expect("Cannot find executable directory");

    let src = Path::new("config.json");
    let dst = exe_dir.join("config.json");

    match fs::copy(&src, &dst) {
        Ok(_) => println!("cargo:warning=Copied config.json → {}", dst.display()),
        Err(e) => println!("cargo:warning=Could NOT copy config.json: {}", e),
    }
}
