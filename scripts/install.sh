#!/bin/bash

# Makes the script fail at the first sign of error
set -euo pipefail
# Sets the script to run in its own directory
cd "$(dirname "$0")/.."

# Where the kernel will be
INSTALL_DIR=/boot/abosbel
# Where the kernel is built
KERNEL=target/x86_64-abosbel-none/release/ab-os-bel

cargo build --release

if ! grub-file --is-x86-multiboot2 "$KERNEL"; then
    echo -e "\e[1;31mERROR:\e[0m Kernel is not Multiboot 2 compliant."
    exit 1
fi

sudo install -Dm644 "$KERNEL" "$INSTALL_DIR/ab-os-bel"
echo "Installed kernel to $INSTALL_DIR/ab-os-bel"

GRUB_ENTRY=scripts/41_abosbel
GRUB_ENTRY_TARGET=/etc/grub.d/41_abosbel

if ! cmp -s "$GRUB_ENTRY" "$GRUB_ENTRY_TARGET"; then
    sudo install -Dm755 "$GRUB_ENTRY" "$GRUB_ENTRY_TARGET"
    echo "GRUB entry changed, regenerating grub.cfg..."
    sudo grub-mkconfig -o /boot/grub/grub.cfg
else
    echo "GRUB entry already up to date."
fi

echo "All done!"