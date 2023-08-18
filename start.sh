EXEC_PATH=$1

mkdir -p target/iso/boot/grub
cp grub.cfg target/iso/boot/grub
cp ${EXEC_PATH} target/iso/boot

grub-mkrescue -o target/ab-os-bel.iso target/iso

qemu-system-x86_64 -cdrom target/ab-os-bel.iso