# Cobalt

View usage statistics of all your apps. Track and determine where you are wasting time, and get your procrastination under control.

> [!WARNING]
> Cobalt was established before in the [archive/main](https://github.com/Enigmatrix/Cobalt/tree/archive/main) branch, and you can try the previous product from the [Github Releases](https://github.com/Enigmatrix/Cobalt/releases).
> This branch is for the next version of Cobalt.

## Docs
See the [ARCHITECTURE.md](./docs/ARCHITECTURE.md) for more information, and the more detailed developer guides 
(general [DEV_GUIDE.md](./docs/DEV_GUIDE.md), then [Engine](./docs/DEV_GUIDE_Engine.md) and [UI](./docs/DEV_GUIDE_UI.md)) for specifics.

## Running

```bash
# Build all
cargo build
# Install UI dependencies
cd src/ui; bun i

# Run the Engine
cargo run --bin engine
# Run the UI
cd src/ui; bun run dev

# Test
cargo test
```