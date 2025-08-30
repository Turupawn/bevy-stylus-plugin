# Bevy Stylus Plugin

A Bevy plugin for integrating with Stylus blockchain contracts.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy-stylus-plugin = { git = "https://github.com/yourusername/bevy-stylus-plugin" }
```

Then in your game:

```rust
use bevy::prelude::*;
use bevy_stylus_plugin::{BlockchainPlugin, BlockchainClient};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BlockchainPlugin)
        .run();
}

fn my_system(blockchain_client: Res<BlockchainClient>) {
    // Use the blockchain client
}
```

## Configuration

Create a `Stylus.toml` file in your project root and set the `PRIVATE_KEY` environment variable.