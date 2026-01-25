#!/bin/bash
set -e

# Build the ELF
cargo build --release

# Run Renode (GUI enabled)
renode renode.resc
