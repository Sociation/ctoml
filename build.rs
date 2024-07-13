use std::env;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::PathBuf;

fn main() {
    if cfg!(unix) {
        let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

        let executable_path = out_dir.join("ctoml");
        let symlink_path = manifest_dir.join("ctoml");

        if symlink_path.exists() {
            fs::remove_file(&symlink_path).unwrap();
        }

        unix_fs::symlink(&executable_path, &symlink_path).unwrap();

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=src/main.rs");
    }
}