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

# Creates the iso
grub-mkrescue -o target/ab-os-bel.iso ${ISO_DIR}



#### QEMU ####
  
# Detect if it's not a test by checking if the executable name ends with "ab-os-bel"
EXTRA_QEMU_FLAGS=""
if [[ $TEST == "true" ]]; then
  EXTRA_QEMU_FLAGS+="-display none "
fi
if [[ $DEBUG == "true" ]]; then
  EXTRA_QEMU_FLAGS+="-s -S "
fi

OVMF=""
for f in \
  "$OVMF_PATH" \
  /usr/share/edk2/x64/OVMF.4m.fd \
  /usr/share/ovmf/OVMF.fd \
  /usr/share/OVMF/OVMF_CODE_4M.fd \
  /usr/share/OVMF/OVMF_CODE.fd \
  /usr/share/edk2/ovmf/OVMF_CODE.fd \
  /usr/share/qemu/OVMF.fd; do
  if [[ -n "$f" && -f "$f" ]]; then OVMF="$f"; break; fi
done
if [[ -z "$OVMF" ]]; then
  echo -e "\e[1;31mERROR:\e[0m OVMF not found. Install it or set OVMF_PATH."
  exit 1
fi


QEMU_FLAGS=''
# Options
QEMU_FLAGS+='-m 1G '
QEMU_FLAGS+='-vga vmware ' # Allows 32 bpp
# Settings
QEMU_FLAGS+='-cdrom target/ab-os-bel.iso '
QEMU_FLAGS+='-serial stdio ' # Allows printing to console
QEMU_FLAGS+='-no-reboot ' # If the os reboots, exit instead
QEMU_FLAGS+='-cpu host ' # Use the host cpu
QEMU_FLAGS+='-enable-kvm ' # Enable KVM
QEMU_FLAGS+="-drive if=pflash,format=raw,readonly=on,file=$OVMF " # Boot with UEFI
# Tests
QEMU_FLAGS+='-device isa-debug-exit,iobase=0xf4,iosize=0x04 '

qemu-system-x86_64 $QEMU_FLAGS $EXTRA_QEMU_FLAGS &

qemu_pid=$!

if [[ $DEBUG == "true" ]]; then
  gdb \
    -ex "target remote localhost:1234" \
    -ex "set architecture i386:x86-64" \
    -ex "hbreak src/main.rs:23" \
    -ex "c" \
    -ex "layout src" \
    $EXEC_PATH
fi

wait $qemu_pid
exit_code=$?

if [ $exit_code -eq 33 ]; then
  exit 0
else
  exit $exit_code
fi
