# Cobalt

View usage statistics of all your apps. Track and determine where you are wasting time, and get your procrastination under control.

> [!WARNING]
> Cobalt was established before in the [archive/main](https://github.com/Enigmatrix/Cobalt/tree/archive/main) branch, and you can try the previous product from the [Github Releases](https://github.com/Enigmatrix/Cobalt/releases).
> This branch is for the next version of Cobalt.

## Docs
See the [ARCHITECTURE.md](./docs/ARCHITECTURE.md) for an overview of the project.

More detailed developer guides:
- General: [DEV_GUIDE.md](./docs/DEV_GUIDE.md)
- Engine: [DEV_GUIDE_Engine.md](./docs/DEV_GUIDE_Engine.md)
- UI: [DEV_GUIDE_UI.md](./docs/DEV_GUIDE_UI.md)

## Running

```bash
# Install UI dependencies
bun i
# Build all
cargo build

# Run the UI
bun dev
# Run the Engine
cargo run --bin engine

# Test
cargo test
```