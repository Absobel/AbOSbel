This is my own OS !
If you want to run it on Windows, install WSL with Debian or whatever and follow the instructions for the distribution in question.

## Features
It boots only on UEFI for now (i think)
It only supports 32 bpp framebuffers for now (so 4 bytes per pixel)  
x86_64 only because I'm not a masochist (at least not for the foreseeable future) (EDIT : funny because building for x86_64 is being a masochist)

## Dependencies

- `rustup`
- `binutils` (for `as`)
- GRUB with EFI support, `xorriso` and `mtools` to build the iso
- QEMU and OVMF (UEFI firmware) to run it
- `gdb` (only for debugging)

| | Arch | Debian / Ubuntu / WSL |
|---|---|---|
| GRUB | `grub` | `grub-common grub-efi-amd64-bin` |
| ISO tools | `libisoburn mtools` | `xorriso mtools` |
| QEMU | `qemu-system-x86` | `qemu-system-x86` |
| OVMF | `edk2-ovmf` | `ovmf` |

On Debian-based systems, without `grub-efi-amd64-bin` and `mtools` the iso silently ends up BIOS-only and you'll land in the UEFI shell instead of booting.

## Installation

To build the project and obtain the kernel binary, run `cargo build`. The kernel binary will be located at `target/x86_64-abosbel-none/release/ab-os-bel` in release mode or `target/x86_64-abosbel-none/debug/ab-os-bel` in debug mode. 

To both build and run the project, use `cargo run`. This will create an `.iso` file located at `target/ab-os-bel.iso` and attempt to launch the OS in QEMU. If QEMU is not installed, the OS won't run, but the `.iso` will still be generated. Also the kernel will be copied to `target/ab-os-bel` for convenience.

If OVMF is installed somewhere `scripts/start.sh` doesn't look, set `OVMF_PATH=/path/to/OVMF.fd`.

If you don't have KVM enabled, enable it, or comment out `QEMU_FLAGS+='-cpu host '` and `QEMU_FLAGS+='-enable-kvm '` in `scripts/start.sh`. However, it'll be slower, so try to enable it.

To debug the project, use `cargo run -- debug`. This will launch QEMU and attach GDB to it. You can setup breakpoints in `scripts/start.sh`

Note 2: Don't bother with `scripts/install.sh`, it will just work on my machine

## Running with podman (or docker)

If you prefer to use podman for building and testing the project, you can do so.
You just need podman (or docker) and then you can build the docker image and run it with the following command :

```bash
podman build -t ab_os_bel . && podman run -it --rm --device /dev/kvm --group-add keep-groups ab_os_bel
```

You will be dropped in a shell inside the docker container and you can run the commands described above. If you want to have a graphical environnement then set it up yourself because it's plateform dependant.
