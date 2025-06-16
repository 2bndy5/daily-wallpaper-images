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

pub struct Notifications(pub HashMap<String, NotificationAlert>);

impl Notifications {
    fn get_pending(&self) -> Vec<String> {
        self.0
            .iter()
            .filter_map(|(key, alert)| {
                if alert.percent < 1.0 {
                    Some(key.to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
    }
}
pub struct NotificationActor {
    notifications: Notifications,
}

impl Actor for NotificationActor {}
impl NotificationActor {
    pub fn new(notification_addr: Address<Self>) -> Self {
        spawn(Self::listen_to_refresh(notification_addr.clone()));
        spawn(Self::listen_to_dismiss(notification_addr.clone()));
        spawn(Self::listen_to_dismiss_all(notification_addr));
        Self {
            notifications: Notifications(HashMap::new()),
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
            debug_print!("dismissing notification");
            // Send a message to the actor.
            let _ = notification_addr.send(dart_signal.message).await;
        }
    }

    async fn listen_to_dismiss_all(mut notification_addr: Address<Self>) {
        let receiver = NotificationDismissAll::get_dart_signal_receiver();
        // Continuously listen for signals.
        while let Some(dart_signal) = receiver.recv().await {
            debug_print!("dismissing notification");
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
        let mut old_key = None;
        for (key, val) in self.notifications.0.iter_mut() {
            if val.title == msg.0.title {
                if val.percent >= 1.0 {
                    old_key = Some(key.to_owned());
                    break;
                }
                val.update(msg.0.clone());
                found = true;
                debug_print!("Updated notification \"{}\" ({})", val.title, val.percent);
                break;
            }
        }
        if let Some(old_key) = old_key {
            let old = self.notifications.0.remove(&old_key).unwrap();
            debug_print!("Removed outdated notification \"{}\"", old.title);
        }
        if !found {
            debug_print!("Adding new notification \"{}\"", msg.0.title);
            let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, false);
            self.notifications.0.insert(timestamp, msg.0);
        }
        NotificationResults {
            notifications: self.notifications.0.clone(),
            pending: self.notifications.get_pending(),
        }
        .send_signal_to_dart();
        Ok(())
    }
}

#[async_trait]
impl Handler<NotificationRefresh> for NotificationActor {
    type Result = Result<()>;

    async fn handle(
        &mut self,
        _msg: NotificationRefresh,
        _context: &MsgContext<Self>,
    ) -> Self::Result {
        NotificationResults {
            notifications: self.notifications.0.clone(),
            pending: self.notifications.get_pending(),
        }
        .send_signal_to_dart();
        debug_print!("Done refreshing notifications");
        Ok(())
    }
}

#[async_trait]
impl Handler<NotificationDismiss> for NotificationActor {
    type Result = Result<()>;

    async fn handle(
        &mut self,
        msg: NotificationDismiss,
        _context: &MsgContext<Self>,
    ) -> Self::Result {
        if let Some(entry) = self.notifications.0.remove_entry(&msg.timestamp) {
            debug_print!("Dismissed \"{}\"", entry.1.title);
            NotificationResults {
                notifications: self.notifications.0.clone(),
                pending: self.notifications.get_pending(),
            }
            .send_signal_to_dart();
        }
        Ok(())
    }
}

#[async_trait]
impl Handler<NotificationDismissAll> for NotificationActor {
    type Result = Result<()>;

    async fn handle(
        &mut self,
        _msg: NotificationDismissAll,
        _context: &MsgContext<Self>,
    ) -> Self::Result {
        self.notifications.0.clear();
        debug_print!("Dismissed all notifications");
        NotificationResults {
            notifications: self.notifications.0.clone(),
            pending: Vec::new(),
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
