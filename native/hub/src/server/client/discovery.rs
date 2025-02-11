use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use log::error;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use discovery::{
    udp_multicast::{DiscoveredDevice, DiscoveryService},
    utils::DeviceInfo,
};

/// Manages persistent storage and in-memory cache of discovered devices.
/// Automatically handles data expiration and file I/O operations.
#[derive(Clone)]
pub struct DiscoveryStore {
    /// Path to the persistent storage file
    path: PathBuf,
    /// In-memory device list with thread-safe access
    devices: Arc<Mutex<Vec<DiscoveredDevice>>>,
}

impl DiscoveryStore {
    /// Creates a new DiscoveryStore instance with the specified base directory.
    /// The actual storage file will be created at `{base_dir}/.discovered`.
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            path: base_path.as_ref().join(".discovered"),
            devices: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Loads devices from persistent storage into memory.
    /// Creates an empty list if the storage file doesn't exist.
    pub async fn load(&self) -> Result<Vec<DiscoveredDevice>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content = tokio::fs::read_to_string(&self.path).await?;
        let devices: Vec<DiscoveredDevice> = toml::from_str(&content)?;
        let devices_clone = devices.clone();
        *self.devices.lock().await = devices;
        Ok(devices_clone)
    }

    /// Persists the current device list to storage, automatically removing
    /// devices that haven't been seen in the last 30 seconds.
    pub async fn save(&self) -> Result<()> {
        let devices = self.devices.lock().await.clone();
        let filtered: Vec<_> = devices
            .iter()
            .filter(|d| d.last_seen.elapsed().unwrap_or(Duration::MAX) < Duration::from_secs(30))
            .cloned()
            .collect();

        let content = toml::to_string(&filtered)?;
        tokio::fs::write(&self.path, content).await?;
        Ok(())
    }

    /// Removes expired devices from both memory and persistent storage
    pub async fn prune_expired(&self) -> Result<()> {
        let mut devices = self.devices.lock().await;
        devices
            .retain(|d| d.last_seen.elapsed().unwrap_or(Duration::MAX) < Duration::from_secs(30));
        self.save().await
    }

    /// Updates or inserts a device into the store and persists changes
    pub async fn update_device(&self, device: DiscoveredDevice) {
        let mut devices = self.devices.lock().await;

        if let Some(existing) = devices
            .iter_mut()
            .find(|d| d.fingerprint == device.fingerprint)
        {
            *existing = device;
        } else {
            devices.push(device);
        }

        if let Err(e) = self.save().await {
            error!("Failed to auto-save device updates: {}", e);
        }
    }

    /// Returns a copy of the current device list
    pub async fn get_devices(&self) -> Vec<DiscoveredDevice> {
        self.devices.lock().await.clone()
    }
}

/// Manages the discovery service lifecycle and coordinates between network operations
/// and device state storage.
pub struct DiscoveryRuntime {
    /// Handle to the discovery service
    service: Arc<DiscoveryService>,
    /// Central device state management
    pub store: DiscoveryStore,
    /// Token for graceful shutdown management
    cancel_token: CancellationToken,
}

impl DiscoveryRuntime {
    /// Initializes a new DiscoveryRuntime with:
    /// - Configuration directory for persistent storage
    /// - Network event channel setup
    /// - Device state loading from storage
    pub async fn new(config_dir: &Path) -> Result<Self> {
        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(100);
        let service = DiscoveryService::new(event_tx);
        let store = DiscoveryStore::new(config_dir);

        // Load persisted devices into memory
        store.load().await?;

        // Start device update listener
        let store_clone = store.clone();
        tokio::spawn(async move {
            while let Some(device) = event_rx.recv().await {
                store_clone.update_device(device).await;
            }
        });

        Ok(Self {
            service: Arc::new(service),
            store,
            cancel_token: CancellationToken::new(),
        })
    }

    /// Starts the discovery service with specified network parameters:
    /// - `device_info`: Local device information to advertise
    /// - `interval`: Broadcast interval for service announcements
    pub async fn start_service(&self, device_info: DeviceInfo, interval: Duration) -> Result<()> {
        self.service
            .listen(device_info.clone(), Some(self.cancel_token.clone()))
            .await?;

        // Start periodic broadcast
        let service_clone = self.service.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = service_clone.announce(device_info.clone()).await {
                    error!("Service announcement failed: {}", e);
                }
                tokio::time::sleep(interval).await;
            }
        });

        Ok(())
    }

    /// Gracefully shuts down the discovery service:
    /// 1. Cancels all ongoing operations
    /// 2. Stops network listeners
    /// 3. Persists final device state
    pub async fn shutdown(&self) {
        self.cancel_token.cancel();
        self.service.shutdown().await;

        if let Err(e) = self.store.save().await {
            error!("Failed to save final device state: {}", e);
        }
    }
}
