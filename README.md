This is my own OS ! And it's written in Rust because I use Rust btw.
There's pretty much no chance it builds in another environnement than Linux but I've never actually tried, but that's what docker is for so yeah.

I'm following this [tutorial](https://os.phil-opp.com/) but changed ~~a few~~ things along the way.

## Installation

To build the project and obtain the kernel binary, run `cargo build`. The kernel binary will be located at `target/x86_64-abosbel-none/release/ab-os-bel` in release mode or `target/x86_64-abosbel-none/debug/ab-os-bel` in debug mode. 

To both build and run the project, use `cargo run`. This will create an `.iso` file located at `target/ab-os-bel.iso` and attempt to launch the OS in QEMU. If QEMU is not installed, the OS won't run, but the `.iso` will still be generated. 

Note: Ensure you have rustup and cargo installed with the nightly toolchain, along with QEMU for running the OS.

## Running with Docker

If you prefer to use Docker for building and testing the project, for example if you don't use linux, you can do so. You first have to install docker if you don't have it ([Docker's official website](https://www.docker.com/products/docker-desktop)).

Then you can build the docker image and run it with the following command (or do whatever you want, idk) :

```bash
docker build -t ab_os_bel . && docker run -it --rm ab_os_bel
```

You will be dropped in a shell inside the docker container and you can run the commands described above. If you want to have a graphical environnement then set it up yourself because it's plateform dependant.
