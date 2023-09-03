This is my own OS ! And it's written in Rust because I use Rust btw.
I don't think it runs on something other than Linux though but never tried it.

I'm following this [tutorial](https://os.phil-opp.com/) but changed ~~a few~~ things along the way.

## Installation

You need to have rustup and cargo installed with the nightly toolchain. You also need to install QEMU the virtual machine that will run the OS.

Then you just need to clone the repo and either run it (with `cargo run`) to launch the OS in QEMU or just build it (`cargo build`) and get the kernel binary in `target/x86_64-abosbel-none/release/ab-os-bel` in release mode or `target/x86_64-abosbel-none/debug/ab-os-bel` in debug mode. For the moment you need to have VGA text mode support on your PC for the OS to display anything.

## Running with Docker

If you prefer to use Docker for building and testing the project, for example if you don't use linux, you can do so. You first have to install docker if you don't have it ([Docker's official website](https://www.docker.com/products/docker-desktop)).

Then you can build the docker image and run it with the following command (or do whatever you want, idk) :

```bash
docker build -t ab_os_bel . && docker run -it --rm ab_os_bel
```

You will be in a shell inside the docker container and you can run the commands described above. If you want to have a graphical environnement then set it up yourself because it's plateform dependant.