#!/bin/bash


### VARS ###

EXEC_PATH=$1

TEST="false"
if [[ $EXEC_PATH != *"ab-os-bel" ]]; then
  TEST="true"
fi
DEBUG="false"
if [[ $2 == "debug" ]]; then
  DEBUG="true"
fi


#### CREATE ISO ####

# VARS
ISO_DIR="target/iso"

cp $EXEC_PATH target/ab-os-bel

# Check if the thins is multiboot compliant
if ! grub-file --is-x86-multiboot2 "$EXEC_PATH"; then
    echo -e "\e[1;31mERROR:\e[0m Kernel is not Multiboot 2 compliant."
    exit 1
fi

# Sets up the iso dir
rm -rf ${ISO_DIR}
mkdir -p ${ISO_DIR}/boot/grub
cp ${EXEC_PATH} ${ISO_DIR}/boot/grub/ab-os-bel
cp grub.cfg ${ISO_DIR}/boot/grub/

mkdir -p ${ISO_DIR}/EFI/BOOT
# Creates BOOTX64.EFI
grub-mkstandalone \
    -O x86_64-efi \
    -o ${ISO_DIR}/EFI/BOOT/BOOTX64.EFI \
    boot/grub/grub.cfg

# Creates the iso
grub-mkrescue -o target/ab-os-bel.iso ${ISO_DIR} > /dev/null 2>&1



#### QEMU ####
  
# Detect if it's not a test by checking if the executable name ends with "ab-os-bel"
EXTRA_QEMU_FLAGS=""
if [[ $TEST == "true" ]]; then
  EXTRA_QEMU_FLAGS+="-display none "
fi
if [[ $DEBUG == "true" ]]; then
  EXTRA_QEMU_FLAGS+="-s -S "
fi

QEMU_FLAGS=''
# Options
QEMU_FLAGS+='-m 1G '
QEMU_FLAGS+='-vga vmware ' # Allows 32 bpp
# Settings
QEMU_FLAGS+='-cdrom target/ab-os-bel.iso '
QEMU_FLAGS+='-serial stdio ' # Allows printing to console
QEMU_FLAGS+='-no-reboot ' # If the os reboots, exist instead
QEMU_FLAGS+='-cpu host ' # Use the host cpu
QEMU_FLAGS+='-enable-kvm ' # Enable KVM
# Tests
QEMU_FLAGS+='-device isa-debug-exit,iobase=0xf4,iosize=0x04 '

qemu-system-x86_64 $QEMU_FLAGS $EXTRA_QEMU_FLAGS &

qemu_pid=$!

if [[ $DEBUG == "true" ]]; then
  rust-gdb \
    -ex "target remote localhost:1234" \
    -ex "set architecture i386:x86-64" \
    -ex "hbreak src/kernel/framebuffer/screen.rs:57" \
    $EXEC_PATH
fi

wait $qemu_pid
exit_code=$?

if [ $exit_code -eq 33 ]; then
  exit 0
else
  exit $exit_code
fi
