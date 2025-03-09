use std::process::Command;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-changed=src/bootstrap/boot.s");
    println!("cargo:rerun-if-changed=src/bootstrap/multiboot.s");

    let out_dir = env::var("OUT_DIR").unwrap();
    
    if !Command::new("as")
        .args(["--64", "-o", &format!("{}/multiboot.o", out_dir), "src/bootstrap/multiboot.s"])
        .status()
        .expect("Failed to execute as command for multiboot.s")
        .success()
    {
        panic!("Failed to compile multiboot.s");
    }
    
    if !Command::new("as")
        .args(["--64", "-o", &format!("{}/boot.o", out_dir), "src/bootstrap/boot.s"])
        .status()
        .expect("Failed to execute as command for boot.s")
        .success()
    {
        panic!("Failed to compile boot.s");
    }

    println!("cargo:rustc-link-arg={}/multiboot.o", out_dir);
    println!("cargo:rustc-link-arg={}/boot.o", out_dir);
}