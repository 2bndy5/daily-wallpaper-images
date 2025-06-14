//! This `hub` crate is the
//! entry point of the Rust logic.

pub(crate) mod bing;
pub(crate) mod common;
pub(crate) mod nasa;
pub(crate) mod notification_center;
#[cfg(not(target_os = "android"))]
pub(crate) mod set_wallpaper;
pub(crate) mod signals;
pub(crate) mod spotlight;
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
    tokio::spawn(notification_center::create_actors());

    // Keep the main function running until Dart shutdown.
    dart_shutdown().await;
}
