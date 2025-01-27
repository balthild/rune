use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use log::error;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use discovery::udp_multicast::{DiscoveredDevice, DiscoveryService};
use discovery::utils::DeviceInfo;

use super::{Broadcaster, DiscoveredDeviceMessage};

pub struct DeviceScanner {
    pub discovery_service: Arc<DiscoveryService>,
    pub broadcast_task: Mutex<Option<JoinHandle<()>>>,
    pub listen_task: Mutex<Option<JoinHandle<()>>>,
    pub devices: Arc<RwLock<HashMap<String, DiscoveredDevice>>>,
    broadcaster: Arc<dyn Broadcaster>,
    is_broadcasting: Arc<AtomicBool>,
}

impl DeviceScanner {
    pub fn new(device_info: DeviceInfo, broadcaster: Arc<dyn Broadcaster>) -> Self {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(100);

        let discovery_service = Arc::new(DiscoveryService::new(device_info, event_tx));

        let scanner = Self {
            discovery_service,
            broadcast_task: Mutex::new(None),
            listen_task: Mutex::new(None),
            devices: Arc::new(RwLock::new(HashMap::new())),
            broadcaster: Arc::clone(&broadcaster),
            is_broadcasting: Arc::new(AtomicBool::new(false)),
        };

        scanner.start_event_forwarder(event_rx);
        scanner
    }

    fn start_event_forwarder(&self, mut event_rx: tokio::sync::mpsc::Receiver<DiscoveredDevice>) {
        let devices = self.devices.clone();
        let broadcaster = self.broadcaster.clone();

        tokio::spawn(async move {
            while let Some(device) = event_rx.recv().await {
                // Update local cache
                let mut devices_map = devices.write().await;
                devices_map.insert(device.fingerprint.clone(), device.clone());

                // Convert to proto message
                let message = DiscoveredDeviceMessage {
                    alias: device.alias,
                    device_model: device.device_model,
                    device_type: device.device_type,
                    fingerprint: device.fingerprint,
                    last_seen_unix_epoch: device
                        .last_seen
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                };

                broadcaster.broadcast(&message);
            }
        });
    }

    pub async fn start_broadcast(&self, duration_seconds: u32) {
        // Terminate existing task first
        self.stop_broadcast().await;

        // Update state
        self.is_broadcasting.store(true, Ordering::SeqCst);

        let discovery_service = self.discovery_service.clone();
        let duration = Duration::from_secs(duration_seconds as u64);
        let is_broadcasting = self.is_broadcasting.clone();

        let task = tokio::spawn(async move {
            let start_time = SystemTime::now();
            loop {
                if let Err(e) = discovery_service.announce().await {
                    error!("Broadcast error: {}", e);
                }

                // Check duration
                if SystemTime::now().duration_since(start_time).unwrap() >= duration {
                    break;
                }

                tokio::time::sleep(Duration::from_secs(3)).await;
            }

            // Update state when task completes
            is_broadcasting.store(false, Ordering::SeqCst);
        });

        *self.broadcast_task.lock().await = Some(task);
    }

    pub async fn stop_broadcast(&self) {
        // Check state to avoid unnecessary operations
        if self.is_broadcasting.load(Ordering::SeqCst) {
            if let Some(task) = self.broadcast_task.lock().await.take() {
                // Graceful shutdown
                if !task.is_finished() {
                    task.abort();
                }
            }
            self.is_broadcasting.store(false, Ordering::SeqCst);
        }
    }

    pub async fn start_listening(&self) {
        let discovery_service = self.discovery_service.clone();

        let task = tokio::spawn(async move {
            if let Err(e) = discovery_service.listen(None).await {
                error!("Listening error: {}", e);
            }
        });

        *self.listen_task.lock().await = Some(task);
    }

    pub async fn stop_listening(&self) {
        if let Some(task) = self.listen_task.lock().await.take() {
            task.abort();
        }
    }

    // Helper method for state checking
    pub fn is_broadcasting(&self) -> bool {
        self.is_broadcasting.load(Ordering::SeqCst)
    }
}
