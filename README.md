# GIF Trick
A [Matricks](https://github.com/wymcg/matricks) plugin that plays any GIF. 
By default, this plugin plays catjam, but the plugin can be [easily modified](#change-the-gif) to play any other GIF.

![giftrick](https://github.com/wymcg/gif_trick/assets/3410869/31d2c778-9e17-4f59-9186-e865adf5dc71)

## Build
- Install the `wasm32-wasi` toolchain by running `rustup target add wasm32-wasi`
- Run `cargo build --release --target wasm32-wasi`
- Run the plugin with [Matricks](https://github.com/wymcg/matricks) (on a Raspberry Pi) or with [Simtricks](https://github.com/wymcg/simtricks) (on other devices).

## Change the GIF
To change the GIF that is compiled into the plugin, you can change the declaration of `GIF_DATA` on line 10 of `lib.rs` to point to a different GIF:

```rust
const GIF_DATA: &[u8] = include_bytes!("../assets/catjam.gif");
                                         // ^---- Change this to a different path and rebuild
```

Once you change the declaration of `GIF_DATA` and rebuild, the new plugin will play the desired GIF.
Be careful about the size of the GIFs you compile into this plugin-- if the GIF is long and/or high-resolution, the plugin can be slow to start up.
