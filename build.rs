fn main() {
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-changed=src/preliminary/boot.s");
    println!("cargo:rerun-if-changed=src/preliminary/multiboot.s");
}