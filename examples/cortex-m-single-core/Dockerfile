# Dockerfile for Cortex-M embedded development with Renode
FROM rust:1.93

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gdb-multiarch \
    gcc-arm-none-eabi \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Install Renode (portable build for arm64)
ARG RENODE_VERSION=1.16.0
RUN mkdir -p /opt/renode \
    && wget -q https://github.com/renode/renode/releases/download/v${RENODE_VERSION}/renode-${RENODE_VERSION}.linux-arm64-portable-dotnet.tar.gz -O /tmp/renode.tar.gz \
    && tar -xzf /tmp/renode.tar.gz -C /opt/renode --strip-components=1 \
    && ln -s /opt/renode/renode /usr/local/bin/renode \
    && rm /tmp/renode.tar.gz

# Add ARM Cortex-M targets
RUN rustup target add thumbv7em-none-eabihf \
    && rustup target add thumbv7m-none-eabi \
    && rustup component add llvm-tools-preview

# Install cargo-binutils for binary manipulation
RUN cargo install cargo-binutils

# Set working directory
WORKDIR /app

# Default command
CMD ["/bin/bash"]
