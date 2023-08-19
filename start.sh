EXEC_PATH=$1

mkdir -p target/iso/boot/grub
cp grub.cfg target/iso/boot/grub
cp ${EXEC_PATH} target/iso/boot

grub-mkrescue -o target/ab-os-bel.iso target/iso --verbose -d /usr/lib/grub/i386-pc

# qemu-system-x86_64 -cdrom target/ab-os-bel.iso

qemu-system-x86_64 \
  -m "4G" \
  -cdrom target/ab-os-bel.iso \
  -monitor stdio \
  -s -S
  #-serial stdio \
