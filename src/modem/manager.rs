use futures::stream::{FuturesUnordered, StreamExt};
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};

use crate::api::SseManager;
use crate::config::SmsStorage;
use crate::db::{Contact, ModemSMS, SimCard};
use crate::webhook;

use super::core::Modem;
use super::types::*;

pub struct ModemManager {
    modems: Arc<RwLock<HashMap<String, Arc<Modem>>>>,
    sim_cards_cache: Arc<RwLock<HashMap<String, SimCard>>>,
    _initialization_semaphore: Arc<Semaphore>,
}

impl ModemManager {
    pub async fn initialize(config: &crate::config::AppConfig) -> anyhow::Result<Self> {
        let initialization_semaphore = Arc::new(Semaphore::new(3));
        let mut initialization_futures = FuturesUnordered::new();

        for (index, device) in config.devices.iter().enumerate() {
            let port = device.com_port.clone();
            let baud_rate = device.baud_rate;
            let sms_storage = device.sms_storage.or(config.settings.sms_storage);
            let temp_device_id = format!("device_{}", index);
            let semaphore = initialization_semaphore.clone();

            initialization_futures.push(async move {
                let _permit = semaphore.acquire().await;
                Self::initialize_single_modem(port, baud_rate, temp_device_id, sms_storage, index)
                    .await
            });
        }

        let mut modems = HashMap::new();
        let mut new_sim_ids = Vec::new();
        let mut failed_count = 0;

        while let Some(result) = initialization_futures.next().await {
            match result {
                Ok((sim_id, modem, is_new)) => {
                    if is_new {
                        new_sim_ids.push(sim_id.clone());
                    }
                    modems.insert(sim_id, Arc::new(modem));
                }
                Err(e) => {
                    error!("Failed to initialize modem: {}", e);
                    failed_count += 1;
                }
            }
        }

        if modems.is_empty() {
            return Err(anyhow::anyhow!("No modems were successfully initialized"));
        }

        info!(
            "Successfully initialized {} modem(s), {} failed",
            modems.len(),
            failed_count
        );

        let manager = Self {
            modems: Arc::new(RwLock::new(modems)),
            sim_cards_cache: Arc::new(RwLock::new(HashMap::new())),
            _initialization_semaphore: initialization_semaphore,
        };

        manager.init_sim_cache().await?;

        if !new_sim_ids.is_empty() {
            manager.init_new_sim_sms_data(new_sim_ids).await;
        }

        Ok(manager)
    }

    async fn initialize_single_modem(
        port: String,
        baud_rate: u32,
        device_id: String,
        sms_storage: Option<SmsStorage>,
        index: usize,
    ) -> anyhow::Result<(String, Modem, bool)> {
        info!("Initializing modem on port {}", port);

        let mut modem = Modem::new(&port, baud_rate, &device_id).await?;

        let pre_sim_id = modem.get_sim_iccid().await.ok().flatten();
        let is_new_sim = if let Some(ref sim_id) = pre_sim_id {
            Self::is_new_sim_id(sim_id).await
        } else {
            false
        };

        modem.init_modem(sms_storage).await?;

        let sim_id = pre_sim_id.unwrap_or_else(|| {
            log::warn!("Using fallback SIM ID for port {}", port);
            format!("fallback_sim_{}", index)
        });

        info!(
            "Successfully initialized modem on {} with SIM ID: {}",
            port, sim_id
        );

        Ok((sim_id, modem, is_new_sim))
    }

    async fn is_new_sim_id(sim_id: &str) -> bool {
        match SimCard::find_by_conditions(Some(sim_id), None, None, None).await {
            Ok(existing) => existing.is_empty(),
            Err(e) => {
                log::warn!("Failed to check SIM ID existence: {}", e);
                true
            }
        }
    }

    async fn init_sim_cache(&self) -> anyhow::Result<()> {
        let modems = self.modems.read().await;
        let sim_ids: Vec<&str> = modems.keys().map(|k| k.as_str()).collect();

        let sim_cards = SimCard::get_by_ids(&sim_ids).await?;

        let mut cache = self.sim_cards_cache.write().await;
        *cache = sim_cards;

        info!("Initialized SIM cache with {} cards", cache.len());
        Ok(())
    }

    async fn init_new_sim_sms_data(&self, new_sim_ids: Vec<String>) {
        let mut futures = FuturesUnordered::new();

        for sim_id in new_sim_ids {
            let modems = self.modems.clone();
            futures.push(async move {
                let modems = modems.read().await;
                if let Some(modem) = modems.get(&sim_id) {
                    match modem.read_sms_sync_insert(SmsType::All).await {
                        Ok(()) => info!("Initialized SMS data for new SIM: {}", sim_id),
                        Err(e) => error!("Failed to initialize SMS data for {}: {}", sim_id, e),
                    }
                }
            });
        }

        while futures.next().await.is_some() {}
    }

    pub async fn get_sim_ids(&self) -> Vec<String> {
        self.modems.read().await.keys().cloned().collect()
    }

    pub async fn get_modem(&self, sim_id: &str) -> Option<Arc<Modem>> {
        self.modems.read().await.get(sim_id).cloned()
    }

    pub async fn send_sms(
        &self,
        sim_id: &str,
        contact: &Contact,
        message: &str,
    ) -> anyhow::Result<(i64, String)> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.send_sms_pdu(contact, message).await
    }

    pub async fn read_sms(&self, sim_id: &str, sms_type: SmsType) -> anyhow::Result<Vec<ModemSMS>> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.read_sms(sms_type).await.map_err(Into::into)
    }

    pub async fn read_sms_async_insert(
        &self,
        sim_id: &str,
        sms_type: SmsType,
        sse_manager: Arc<SseManager>,
        webhook_manager: Option<webhook::WebhookManager>,
    ) -> anyhow::Result<()> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem
            .read_sms_async_insert(sms_type, sse_manager, webhook_manager)
            .await
    }

    pub async fn read_sms_sync_insert(
        &self,
        sim_id: &str,
        sms_type: SmsType,
    ) -> anyhow::Result<()> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.read_sms_sync_insert(sms_type).await
    }

    pub async fn read_all_sms_async(
        &self,
        sms_type: SmsType,
        sse_manager: Arc<SseManager>,
        webhook_manager: Option<webhook::WebhookManager>,
    ) {
        let modems = self.modems.read().await;
        let mut futures = FuturesUnordered::new();

        for (sim_id, modem) in modems.iter() {
            let modem = modem.clone();
            let sse_manager = sse_manager.clone();
            let webhook_manager = webhook_manager.clone();
            let sim_id = sim_id.clone();

            futures.push(async move {
                if let Err(e) = modem
                    .read_sms_async_insert(sms_type, sse_manager, webhook_manager)
                    .await
                {
                    error!("Failed to read SMS for {}: {}", sim_id, e);
                }
            });
        }

        while futures.next().await.is_some() {}
    }

    pub async fn get_signal_quality(&self, sim_id: &str) -> anyhow::Result<Option<SignalQuality>> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.get_signal_quality().await.map_err(Into::into)
    }

    pub async fn check_network_registration(
        &self,
        sim_id: &str,
    ) -> anyhow::Result<Option<NetworkRegistrationStatus>> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.check_network_registration().await.map_err(Into::into)
    }

    pub async fn check_operator(&self, sim_id: &str) -> anyhow::Result<Option<OperatorInfo>> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.check_operator().await.map_err(Into::into)
    }

    pub async fn get_modem_model(&self, sim_id: &str) -> anyhow::Result<Option<ModemInfo>> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.get_modem_model().await.map_err(Into::into)
    }

    pub async fn get_sms_center(&self, sim_id: &str) -> anyhow::Result<Option<String>> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.get_sms_center().await.map_err(Into::into)
    }

    pub async fn get_network_info(&self, sim_id: &str) -> anyhow::Result<Option<String>> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.get_network_info().await.map_err(Into::into)
    }

    pub async fn get_sim_status(&self, sim_id: &str) -> anyhow::Result<Option<String>> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.get_sim_status().await.map_err(Into::into)
    }

    pub async fn get_memory_status(&self, sim_id: &str) -> anyhow::Result<Option<String>> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.get_memory_status().await.map_err(Into::into)
    }

    pub async fn get_temperature_info(&self, sim_id: &str) -> anyhow::Result<Option<String>> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.get_temperature_info().await.map_err(Into::into)
    }

    pub async fn set_sms_storage(
        &self,
        sim_id: &str,
        sms_storage: SmsStorage,
    ) -> anyhow::Result<()> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.set_sms_storage(sms_storage).await.map_err(Into::into)
    }

    pub async fn get_sms_storage_status(&self, sim_id: &str) -> anyhow::Result<Option<String>> {
        let modem = self
            .get_modem(sim_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Modem not found for SIM ID: {}", sim_id))?;

        modem.get_sms_storage_status().await.map_err(Into::into)
    }

    pub async fn get_sim_card_cached(&self, sim_id: &str) -> Option<SimCard> {
        self.sim_cards_cache.read().await.get(sim_id).cloned()
    }

    pub async fn update_sim_cache(&self, sim_card: SimCard) {
        let mut cache = self.sim_cards_cache.write().await;
        cache.insert(sim_card.id.clone(), sim_card);
    }
}