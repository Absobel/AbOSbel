This is my own OS ! And it's written in Rust because I use Rust btw.
I don't think it runs on something other than Linux though but never tried it.

I'm following this [tutorial](https://os.phil-opp.com/) but changed a few things along the way.

## Installation

You obvioulsy must have rustup and cargo installed. But you also need to install QEMU the virtual machine that will run the OS.
It is also possible to just download the binary file and run it on your computer but it's not recommended as I can't test it. 
It will work only if your computer is compatible with legacy BIOS anyway.

Then you just need to clone the repo and run it.

```bash
git clone https://github.com/Absobel/AbOSbel.git     
cd AbOSbel
cargo run
```