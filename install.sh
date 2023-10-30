MOUNTPOINT=/tmp/mnt

cargo build --release -q

mkdir -p $MOUNTPOINT
sudo mount /dev/nvme0n1p4 $MOUNTPOINT
sudo rm -rf $MOUNTPOINT/*
sudo cp target/x86_64-abosbel-none/release/ab-os-bel $MOUNTPOINT
sudo umount $MOUNTPOINT
