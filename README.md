This is my own OS ! And it's written in Rust because I use Rust btw.
I don't think it runs on something other than Linux though but never tried it.

I'm following this [tutorial](https://os.phil-opp.com/) but changed ~~a few~~ things along the way.

## Installation

You need to have rustup and cargo installed with the nightly toolchain. You also need to install QEMU the virtual machine that will run the OS.

Then you just need to clone the repo and either run it (with `cargo run`) to launch the OS in QEMU or just build it and get the kernel binary in `target/x86_64-abosbel-none/release/ab-os-bel` in release mode or `target/x86_64-abosbel-none/debug/ab-os-bel` in debug mode. For the moment you need to have VGA text mode support for the os to display anything.