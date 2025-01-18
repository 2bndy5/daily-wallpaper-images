//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.
#![cfg(not(target_os = "android"))]
use crate::messages::*;
use anyhow::Result;
use messages::prelude::{async_trait, Actor, Address, Context as MsgContext, Handler};
use rinf::debug_print;
use tokio::spawn;

// The actor that holds the counter state and handles messages.
pub struct WallpaperActor;

// Implementing the `Actor` trait for `CountingActor`.
// This defines `CountingActor` as an actor in the async system.
impl Actor for WallpaperActor {}

impl WallpaperActor {
    pub fn new(setter_addr: Address<Self>) -> Self {
        spawn(Self::listen_to_set_wallpaper_trigger(setter_addr));
        WallpaperActor {}
    }

    async fn listen_to_set_wallpaper_trigger(mut setter_addr: Address<Self>) {
        // Spawn an asynchronous task to listen for
        // button click signals from Dart.
        let receiver = SetWallpaper::get_dart_signal_receiver();
        // Continuously listen for signals.
        while let Some(dart_signal) = receiver.recv().await {
            debug_print!("setting image as wallpaper");
            // Send a message to the actor.
            let _ = setter_addr.send(dart_signal.message).await;
        }
    }
}

#[async_trait]
impl Handler<SetWallpaper> for WallpaperActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(&mut self, msg: SetWallpaper, _context: &MsgContext<Self>) -> Self::Result {
        let selection = match msg.selected {
            Some(s) => s,
            None => return Ok(()),
        };
        debug_print!(
            "Setting wallpaper to {} (mode: {:?})",
            selection.path,
            selection.mode()
        );
        let mode = match &selection.mode() {
            WallpaperMode::Center => more_wallpapers::Mode::Center,
            WallpaperMode::Crop => more_wallpapers::Mode::Crop,
            WallpaperMode::Fit => more_wallpapers::Mode::Fit,
            WallpaperMode::Stretch => more_wallpapers::Mode::Stretch,
            WallpaperMode::Tile => more_wallpapers::Mode::Tile,
        };
        if let Err(e) = more_wallpapers::set_wallpapers_from_vec(vec![&selection.path], &selection.path, mode) {
            debug_print!("Failed to set the desktop wallpaper: {e:?}");
        }

        Ok(())
    }
}

// Creates and spawns the actors in the async system.
pub async fn create_actors() -> Result<()> {
    // Create actor contexts.
    let wallpaper_context = MsgContext::new();
    let wall_addr = wallpaper_context.address();

    // Spawn actors.
    let actor = WallpaperActor::new(wall_addr);
    spawn(wallpaper_context.run(actor));
    Ok(())
}
