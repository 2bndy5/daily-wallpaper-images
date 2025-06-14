//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.
#![cfg(not(target_os = "android"))]

use crate::{
    common::check_err,
    notification_center::{NotificationActor, NotificationUpdate},
    signals::{NotificationAlert, NotificationSeverity, SetWallpaper, WallpaperMode},
};
use anyhow::{anyhow, Result};
use messages::prelude::{async_trait, Actor, Address, Context as MsgContext, Handler};
use rinf::{debug_print, DartSignal};
use tokio::spawn;
use wallpaper::{DesktopClient, DesktopWallpaper};

// The actor that holds the counter state and handles messages.
pub struct WallpaperActor;

// Implementing the `Actor` trait for `CountingActor`.
// This defines `CountingActor` as an actor in the async system.
impl Actor for WallpaperActor {}

impl WallpaperActor {
    pub fn new(
        setter_addr: Address<Self>,
        notification_center: Address<NotificationActor>,
    ) -> Self {
        spawn(Self::listen_to_set_wallpaper_trigger(
            setter_addr,
            notification_center,
        ));
        Self {}
    }

    async fn listen_to_set_wallpaper_trigger(
        mut setter_addr: Address<Self>,
        mut notification_center: Address<NotificationActor>,
    ) {
        // Spawn an asynchronous task to listen for
        // button click signals from Dart.
        let receiver = SetWallpaper::get_dart_signal_receiver();
        // Continuously listen for signals.
        while let Some(dart_signal) = receiver.recv().await {
            debug_print!("setting image as wallpaper");
            // Send a message to the actor.
            let notification = match setter_addr.send(dart_signal.message).await {
                Err(e) => NotificationAlert {
                    title: "Failed to set the desktop wallpaper".to_string(),
                    body: format!("{e:?}"),
                    percent: 1.0,
                    severity: NotificationSeverity::Error,
                    status_message: String::new(),
                },
                Ok(result) => match result {
                    Ok(_) => NotificationAlert {
                        title: "Set the desktop wallpaper".to_string(),
                        body: "Success".to_string(),
                        percent: 1.0,
                        severity: NotificationSeverity::Info,
                        status_message: String::new(),
                    },
                    Err(e) => NotificationAlert {
                        title: "Failed to set the desktop wallpaper".to_string(),
                        body: format!("{e:?}"),
                        percent: 1.0,
                        severity: NotificationSeverity::Error,
                        status_message: String::new(),
                    },
                },
            };
            if let Ok(result) = check_err(
                notification_center
                    .send(NotificationUpdate(notification))
                    .await
                    .map_err(|e| anyhow!("Failed to send notification: {e:?}")),
            ) {
                let _ = check_err(result);
            }
        }
    }
}

#[async_trait]
impl Handler<SetWallpaper> for WallpaperActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(&mut self, msg: SetWallpaper, _context: &MsgContext<Self>) -> Self::Result {
        let selection = msg.selected;
        debug_print!(
            "Setting wallpaper to {} (mode: {:?})",
            selection.path,
            selection.mode
        );
        let mode = match &selection.mode {
            WallpaperMode::Center => wallpaper::Mode::Center,
            WallpaperMode::Crop => wallpaper::Mode::Crop,
            WallpaperMode::Fit => wallpaper::Mode::Fit,
            WallpaperMode::Stretch => wallpaper::Mode::Stretch,
            WallpaperMode::Tile => wallpaper::Mode::Tile,
        };
        let mut client = check_err(
            DesktopWallpaper::new()
                .map_err(|e| anyhow!("Failed to init desktop wallpaper client: {e:?}")),
        )?;
        client
            .set_wallpaper(&selection.path, mode)
            .map_err(|e| anyhow!("Failed to set wallpaper {}: {e:?}", selection.path))
    }
}

// Creates and spawns the actors in the async system.
pub async fn create_actors(notification_center: Address<NotificationActor>) -> Result<()> {
    // Create actor contexts.
    let wallpaper_context = MsgContext::new();
    let wall_addr = wallpaper_context.address();

    // Spawn actors.
    let actor = WallpaperActor::new(wall_addr, notification_center);
    spawn(wallpaper_context.run(actor));
    Ok(())
}
