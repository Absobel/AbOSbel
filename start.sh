#!/bin/bash

EXEC_PATH=$1

# Check if the thins is multiboot compliant
if grub-file --is-x86-multiboot2 "$EXEC_PATH"; then
    :
else
    echo -e "\e[1;31mERROR:\e[0m Kernel is not Multiboot 2 compliant."
    exit 1
fi

# BIOS
mkdir -p target/iso/boot/grub
cp grub.cfg target/iso/boot/grub
rm -f target/iso/boot/ab-os-bel*
mv ${EXEC_PATH} target/iso/boot/ab-os-bel

# UEFI
mkdir -p target/iso/EFI/BOOT/
grub-mkimage -o BOOTX64.EFI -p /EFI/BOOT -O x86_64-efi
mv BOOTX64.EFI target/iso/EFI/BOOT/

grub-mkrescue -o target/ab-os-bel.iso target/iso -d /usr/lib/grub/i386-pc > /dev/null 2>&1

# Detect if it's not a test by checking if the executable name ends with "ab-os-bel"
if [[ $EXEC_PATH == *"ab-os-bel" ]]; then
  EXTRA_QEMU_FLAGS=""
else
  EXTRA_QEMU_FLAGS="-display none"
fi

qemu-system-x86_64 \
  -m "500M" \
  -cdrom target/ab-os-bel.iso \
  -serial stdio \
  -no-reboot \
  -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
  # -drive if=pflash,format=raw,unit=0,file=/usr/share/ovmf/x64/OVMF_CODE.fd,readonly=on \
  # -drive if=pflash,format=raw,unit=1,file=/usr/share/ovmf/x64/OVMF_VARS.fd \
  # -net none
  $EXTRA_QEMU_FLAGS
  #-d int,cpu_reset \
  #-s -S

exit_code=$?

if [ $exit_code -eq 33 ]; then
  exit 0
else
  exit $exit_code
fi
