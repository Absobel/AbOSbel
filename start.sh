#!/bin/bash

EXEC_PATH=$1

# Check if the thins is multiboot compliant
if ! grub-file --is-x86-multiboot2 "$EXEC_PATH"; then
    echo -e "\e[1;31mERROR:\e[0m Kernel is not Multiboot 2 compliant."
    exit 1
fi

ISO_DIR="target/iso"

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

# Detect if it's not a test by checking if the executable name ends with "ab-os-bel"
if [[ $EXEC_PATH == *"ab-os-bel" ]]; then
  EXTRA_QEMU_FLAGS=""
else
  EXTRA_QEMU_FLAGS="-display none"
fi

FLAGS=''

# Options
FLAGS+='-m 1G '
FLAGS+='-vga vmware ' # Allows 32 bpp

FLAGS+='-cdrom target/ab-os-bel.iso '
FLAGS+='-serial stdio ' # Allows printing to console
FLAGS+='-no-reboot ' # If the os reboots, exist instead

# Tests
FLAGS+='-device isa-debug-exit,iobase=0xf4,iosize=0x04 '

# UEFI
FLAGS+='-drive if=pflash,format=raw,unit=0,file=/usr/share/ovmf/x64/OVMF_CODE.fd,readonly=on '
FLAGS+='-drive if=pflash,format=raw,unit=1,file=/usr/share/ovmf/x64/OVMF_VARS.fd '

qemu-system-x86_64 \
  $FLAGS \
  $EXTRA_QEMU_FLAGS

exit_code=$?

if [ $exit_code -eq 33 ]; then
  exit 0
else
  exit $exit_code
fi
