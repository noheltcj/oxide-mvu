#!/bin/bash
set -e

echo "==============================="
echo "Cortex-M Single-Core Example"
echo "==============================="
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Error: Docker is not running. Please start Docker and try again."
    exit 1
fi

echo "Building Docker image..."
docker build -t oxide-mvu-cortex-m .
echo ""

echo "Running container and example app..."
echo ""

docker run --rm -it \
    --name oxide-mvu-cortex-m-example \
    -v "$(pwd)/../..:/app" \
    oxide-mvu-cortex-m bash -c "cd examples/cortex-m-single-core && ./run.sh"

echo ""
