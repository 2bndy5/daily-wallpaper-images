use crate::{
    services::ImageServiceActor,
    signals::{WallpaperMode, WallpaperModeCache},
};
use anyhow::{Context, Result};
use messages::prelude::{async_trait, Address, Context as MsgContext, Handler};
use rinf::{debug_print, DartSignal, RustSignal};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    mode: WallpaperMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: WallpaperMode::Fit,
        }
    }
}

impl ImageServiceActor {
    pub async fn listen_to_wallpaper_mode(mut service_addr: Address<Self>) {
        let receiver = WallpaperModeCache::get_dart_signal_receiver();
        while let Some(dart_signal) = receiver.recv().await {
            let _ = service_addr.send(dart_signal.message).await;
        }
    }
}

#[async_trait]
impl Handler<WallpaperModeCache> for ImageServiceActor {
    type Result = Result<()>;

    async fn handle(
        &mut self,
        msg: WallpaperModeCache,
        _context: &MsgContext<Self>,
    ) -> Self::Result {
        let cached_settings = self.app_cache_dir.join("settings.json");

        let mut settings = if let Some(mode) = msg.mode {
            debug_print!("Setting mode to {mode:?}");
            Settings { mode }
        } else {
            Settings::default()
        };

        // if settings' cache does not exist or the mode is set with Some value
        if !cached_settings.exists() || msg.mode.is_some() {
            fs::write(
                &cached_settings,
                serde_json::to_string(&settings)
                    .with_context(|| "Failed to convert settings data to JSON")?,
            )
            .await?;
        }

        // if mode is not set (None), then get the value from cache and send it back to dart
        if msg.mode.is_none() {
            debug_print!("Getting WallpaperMode from cache");
            let cached_data = fs::read_to_string(cached_settings).await?;
            settings = serde_json::from_str(&cached_data)
                .with_context(|| "Failed to deserialize cached settings")?;
        }
        let result = WallpaperModeCache {
            mode: Some(settings.mode),
        };
        result.send_signal_to_dart();
        Ok(())
    }
}
