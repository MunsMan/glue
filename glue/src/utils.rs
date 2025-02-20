use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use log::info;
use notify_rust::Notification;
use tokio::sync::Notify;

#[derive(Clone, Debug)]
pub struct CancelableTimer {
    duration: Duration,
    notify: Arc<Notify>,
    is_canceled: Arc<Mutex<bool>>,
}

impl CancelableTimer {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            notify: Arc::new(Notify::new()),
            is_canceled: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&self) {
        let notify = self.notify.clone();
        let is_canceled = self.is_canceled.clone();
        let duration = self.duration;

        tokio::spawn(async move {
            tokio::select! {
                _ = tokio::time::sleep(duration) => {
                    if !*is_canceled.lock().unwrap() {
                        Notification::new()
                            .summary("Coffee still required?")
                            .body(&format!("The System is coffeinated since {:#?}", duration))
                            .show()
                            .unwrap();
                    }
                }
                _ = notify.notified() => {
                    info!("Timer was canceled!");
                }
            };
        });
    }

    pub fn cancel(&self) {
        *self.is_canceled.lock().unwrap() = true;
        self.notify.notify_one();
    }
}
