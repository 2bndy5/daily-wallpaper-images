//! This `hub` crate is the
//! entry point of the Rust logic.

mod bing;
mod common;
mod nasa;
mod set_wallpaper;
mod signals;
mod spotlight;
use rinf::{dart_shutdown, write_interface};

// Uncomment below to target the web.
// use tokio_with_wasm::alias as tokio;

write_interface!();

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Spawn concurrent tasks.
    // Always use non-blocking async functions like `tokio::fs::File::open`.
    // If you must use blocking code, use `tokio::task::spawn_blocking`
    // or the equivalent provided by your async library.
    tokio::spawn(bing::create_actors());
    tokio::spawn(nasa::create_actors());
    tokio::spawn(spotlight::create_actors());
    #[cfg(not(target_os = "android"))]
    tokio::spawn(set_wallpaper::create_actors());

    // Keep the main function running until Dart shutdown.
    dart_shutdown().await;
}
