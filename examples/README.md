# oxide-mvu Examples

This directory contains example projects demonstrating `oxide-mvu` in various environments and use cases.

## Available Examples

### Embedded Examples

#### [cortex-m-single-core](./cortex-m-single-core)

Demonstrates `oxide-mvu` running in a `no_std` environment on a single-core ARM Cortex-M microcontroller, emulated via Renode.

- **Platform**: nRF52840 (ARM Cortex-M4F, thumbv7em-none-eabihf)
- **Emulator**: Renode nrf52840 platform
- **Features**: `no_std`, embassy-nrf HAL, concurrent async event sources
- **Status**: Complete MVU application with timer and button simulation

**Quick start**:
```bash
cd examples/cortex-m-single-core
./test-docker.sh  # Runs in a Docker container with all dependencies isolated
```

## Planned Examples

- **cortex-m-multi-core**: Multi-core Cortex-M demonstrating concurrent event processing
- **wasm-browser**: WebAssembly example running oxide-mvu in the browser

## Contributing

When adding new examples, please:

1. Create a descriptive directory name
2. Include a comprehensive README.md
3. Provide Docker support when external dependencies are required
4. Include build and run scripts for ease of use
5. Update this index with a brief description

For questions or suggestions, please open an issue.
