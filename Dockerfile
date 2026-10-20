# Use a specific base image
  FROM docker.io/library/debian:bookworm-slim

# Metadata as labels
LABEL maintainer="Absobel" \
      version="1.0" \
      description="Environment for building and testing AbOSbel for those who don't use the same machine as me... Losers."

# Install basic and required packages
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       git curl ca-certificates \
       binutils gcc libc6-dev \
       grub-common grub-efi-amd64-bin grub-pc-bin \
       xorriso mtools \
       qemu-system-x86 ovmf \
    && rm -rf /var/lib/apt/lists/*

# Install Rust nightly and set it up
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y \
    && /root/.cargo/bin/rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

# Clone the repository
RUN git clone https://github.com/Absobel/AbOSbel.git /home/AbOSbel

# Set ENV and WORKDIR
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /home/AbOSbel