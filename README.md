This is my own OS ! And it's written in Rust because I use Rust btw.
There's pretty much no chance it builds in another environnement than Linux but I've never actually tried, but that's what docker is for so yeah.

## Features
It boots on UEFI so too bad for the three BIOS users out there  
It only supports 32 bpp framebuffers for now (so 4 bytes per pixel)  
x86_64 only because I'm not a masochist (at least not for the foreseeable future) (EDIT : funny because building for x86_64 is being a masochist)

## Installation

To build the project and obtain the kernel binary, run `cargo build`. The kernel binary will be located at `target/x86_64-abosbel-none/release/ab-os-bel` in release mode or `target/x86_64-abosbel-none/debug/ab-os-bel` in debug mode. 

To both build and run the project, use `cargo run`. This will create an `.iso` file located at `target/ab-os-bel.iso` and attempt to launch the OS in QEMU. If QEMU is not installed, the OS won't run, but the `.iso` will still be generated. Also the kernel will be copied to `target/ab-os-bel` for convenience.

To debug the project, use `cargo run -- debug`. This will launch QEMU and attach GDB to it. You can setup breakpoints in `scripts/start.sh`

Note: Ensure you have rustup and cargo installed with the nightly toolchain, along with QEMU for running the OS, the `libisoburn` library and the `mtools` package to create the iso.
Note 2: Don't bother with `scripts/install.sh`, it will just work on my machine

## Running with Docker

If you prefer to use Docker for building and testing the project, for example if you don't use linux, you can do so. You first have to install docker if you don't have it ([Docker's official website](https://www.docker.com/products/docker-desktop)).

Then you can build the docker image and run it with the following command (or do whatever you want, idk) :

```bash
docker build -t ab_os_bel . && docker run -it --rm ab_os_bel
```

You will be dropped in a shell inside the docker container and you can run the commands described above. If you want to have a graphical environnement then set it up yourself because it's plateform dependant.
