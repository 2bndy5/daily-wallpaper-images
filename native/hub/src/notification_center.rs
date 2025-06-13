use std::collections::HashMap;

#[cfg(not(target_os = "android"))]
use crate::set_wallpaper;
use crate::{bing, nasa, signals::notifications::*, spotlight};
use anyhow::Result;
use chrono::{SecondsFormat, Utc};
use messages::prelude::{async_trait, Actor, Address, Context as MsgContext, Handler};
use rinf::{debug_print, DartSignal, RustSignal};
use tokio::spawn;

pub struct NotificationUpdate(pub NotificationAlert);

pub struct NotificationActor {
    notifications: HashMap<String, NotificationAlert>,
}

impl Actor for NotificationActor {}
impl NotificationActor {
    pub fn new(notification_addr: Address<Self>) -> Self {
        spawn(Self::listen_to_refresh(notification_addr.clone()));
        spawn(Self::listen_to_dismiss(notification_addr));
        Self {
            notifications: HashMap::new(),
        }
    }

    async fn listen_to_refresh(mut notification_addr: Address<Self>) {
        let receiver = NotificationRefresh::get_dart_signal_receiver();
        // Continuously listen for signals.
        while let Some(dart_signal) = receiver.recv().await {
            debug_print!("refreshing notifications");
            // Send a message to the actor.
            let _ = notification_addr.send(dart_signal.message).await;
        }
    }

    async fn listen_to_dismiss(mut notification_addr: Address<Self>) {
        let receiver = NotificationDismiss::get_dart_signal_receiver();
        // Continuously listen for signals.
        while let Some(dart_signal) = receiver.recv().await {
            debug_print!("refreshing notifications");
            // Send a message to the actor.
            let _ = notification_addr.send(dart_signal.message).await;
        }
    }
}

#[async_trait]
impl Handler<NotificationUpdate> for NotificationActor {
    type Result = Result<()>;
    async fn handle(
        &mut self,
        msg: NotificationUpdate,
        _context: &MsgContext<Self>,
    ) -> Self::Result {
        let mut found = false;
        for (_, val) in self.notifications.iter_mut() {
            if val.title == msg.0.title {
                val.update(msg.0.clone());
                found = true;
                break;
            }
        }
        if !found {
            self.notifications.insert(
                Utc::now().to_rfc3339_opts(SecondsFormat::Millis, false),
                msg.0,
            );
        }
        NotificationResults {
            notifications: self.notifications.clone(),
        }
        .send_signal_to_dart();
        Ok(())
    }
}

#[async_trait]
impl Handler<NotificationRefresh> for NotificationActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(
        &mut self,
        _msg: NotificationRefresh,
        _context: &MsgContext<Self>,
    ) -> Self::Result {
        NotificationResults {
            notifications: self.notifications.clone(),
        }
        .send_signal_to_dart();
        Ok(())
    }
}

#[async_trait]
impl Handler<NotificationDismiss> for NotificationActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(
        &mut self,
        msg: NotificationDismiss,
        _context: &MsgContext<Self>,
    ) -> Self::Result {
        self.notifications.remove_entry(&msg.timestamp);
        NotificationResults {
            notifications: self.notifications.clone(),
        }
        .send_signal_to_dart();
        Ok(())
    }
}

// Creates and spawns the actors in the async system.
pub async fn create_actors() -> Result<()> {
    // Create actor contexts.
    let notification_context = MsgContext::new();
    let notification_addr = notification_context.address();

    // Spawn actors.
    let actor = NotificationActor::new(notification_addr.clone());
    spawn(notification_context.run(actor));

    // now spawn sub tasks that can send messages to this task
    spawn(bing::create_actors(notification_addr.clone()));
    spawn(nasa::create_actors(notification_addr.clone()));
    spawn(spotlight::create_actors(notification_addr.clone()));
    #[cfg(not(target_os = "android"))]
    {
        spawn(set_wallpaper::create_actors(notification_addr.clone()));
    }
    Ok(())
}
