use alloy::providers::{ProviderBuilder, RootProvider, WsConnect};
use alloy::signers::local::PrivateKeySigner;
use eyre::Result;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Build a public read-only WebSocket provider from a ws/wss url.
pub type WsProvider = RootProvider;

pub async fn ws_public(url: &str) -> Result<WsProvider> {
    let ws = WsConnect::new(url);
    let provider = ProviderBuilder::new()
        .disable_recommended_fillers()
        .connect_ws(ws)
        .await?;
    Ok(provider)
}

/// Build a WebSocket provider with a private key signer for sending transactions.
pub async fn ws_with_private_key(
    url: String,
    pk_hex: String,
) -> Result<impl alloy::providers::Provider + Clone + Send + Sync + 'static> {
    let signer = PrivateKeySigner::from_str(&pk_hex)?;
    let ws = WsConnect::new(url);
    let provider = ProviderBuilder::new()
        .disable_recommended_fillers()
        .wallet(signer)
        .connect_ws(ws)
        .await?;
    Ok(provider)
}

pub struct ProviderPool {
    ws_listener: WsProvider,
    ws_reader: WsProvider,
}

static POOL: OnceLock<ProviderPool> = OnceLock::new();

/// Initialize global provider pool from current config.
pub async fn init_pool() -> Result<&'static ProviderPool> {
    let url = crate::config::get().ws_rpc_url.clone();
    let listener = ws_public(&url).await?;
    let reader = ws_public(&url).await?;
    let pool = ProviderPool {
        ws_listener: listener,
        ws_reader: reader,
    };
    let _ = POOL.set(pool);
    Ok(POOL.get().expect("provider pool must be initialized"))
}

pub fn get_pool() -> &'static ProviderPool {
    POOL.get()
        .expect("provider pool is not initialized; call init_pool() first")
}

impl ProviderPool {
    pub fn ws_listener(&self) -> WsProvider {
        self.ws_listener.clone()
    }
    pub fn ws_reader(&self) -> WsProvider {
        self.ws_reader.clone()
    }
}

/// Demo: send a write transaction to ShowManager.updateShow (requires signer provider)
#[allow(unused)]
pub async fn demo_update_show_name<
    P: alloy::providers::Provider + Clone + Send + Sync + 'static,
>(
    provider: P,
    show_id: alloy::primitives::U256,
    name: String,
    metadata_uri: String,
) -> Result<()> {
    let addr = crate::config::get().addresses.show_manager;
    let inst = crate::contract::bindings::ShowManager::ShowManagerInstance::new(
        addr, provider,
    );
    let _pending = inst.updateShow(show_id, name, metadata_uri).send().await?;
    Ok(())
}

/// SignerPool: manage named private keys and build signer-enabled providers on demand.
pub struct SignerPool {
    // name -> private key hex (can be 0x-prefixed)
    keys: RwLock<HashMap<String, String>>,
}

static SIGNER_POOL: OnceLock<SignerPool> = OnceLock::new();

impl SignerPool {
    fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
        }
    }

    /// Register a named private key (override if exists). Returns error if key is invalid.
    pub fn register(
        &self,
        name: impl Into<String>,
        pk_hex: impl Into<String>,
    ) -> Result<()> {
        let name = name.into();
        let pk_hex = pk_hex.into();
        // Validate by attempting to parse
        let _ = PrivateKeySigner::from_str(&pk_hex)?;
        let mut map = self
            .keys
            .write()
            .expect("SignerPool RwLock poisoned while registering");
        map.insert(name, pk_hex);
        Ok(())
    }

    /// List registered signer names.
    pub fn names(&self) -> Vec<String> {
        self.keys
            .read()
            .expect("SignerPool RwLock poisoned while reading")
            .keys()
            .cloned()
            .collect()
    }

    /// Remove a signer by name. Returns true if existed.
    pub fn unregister(&self, name: &str) -> bool {
        let mut map = self
            .keys
            .write()
            .expect("SignerPool RwLock poisoned while unregistering");
        map.remove(name).is_some()
    }

    /// Clear all signers in memory.
    pub fn clear(&self) {
        let mut map = self
            .keys
            .write()
            .expect("SignerPool RwLock poisoned while clearing");
        map.clear();
    }

    fn get_pk(&self, name: &str) -> Result<String> {
        let map = self
            .keys
            .read()
            .expect("SignerPool RwLock poisoned while getting pk");
        let pk = map.get(name).cloned().ok_or_else(|| {
            eyre::eyre!(format!("no signer named '{}'", name))
        })?;
        Ok(pk)
    }

    /// Build a signer-enabled provider for the given named account on demand.
    pub async fn provider_for(
        &self,
        name: &str,
    ) -> Result<impl alloy::providers::Provider + Clone + Send + Sync + 'static>
    {
        let pk_hex: String = self.get_pk(name)?;
        let url: String = crate::config::get().ws_rpc_url.clone();
        ws_with_private_key(url, pk_hex).await
    }

    /// Convenience: build provider for the "default" signer.
    pub async fn default_provider(
        &self,
    ) -> Result<impl alloy::providers::Provider + Clone + Send + Sync + 'static>
    {
        self.provider_for("default").await
    }
}

/// Initialize global SignerPool and load PRIVATE_KEY from env as "default" signer (if present).
pub fn init_signer_pool_from_env() -> Result<&'static SignerPool> {
    let pool = SIGNER_POOL.get_or_init(|| SignerPool::new());
    if let Ok(pk) = std::env::var("PRIVATE_KEY") {
        pool.register("default", pk)?;
    }
    Ok(pool)
}

/// Initialize global SignerPool by loading from disk first, then env.
pub fn init_signer_pool_from_env_and_disk() -> Result<&'static SignerPool> {
    let pool = SIGNER_POOL.get_or_init(|| SignerPool::new());
    let path = default_signers_store_path();
    let _ = pool.load_from_file(&path);
    if let Ok(pk) = std::env::var("PRIVATE_KEY") {
        pool.register("default", pk)?;
    }
    Ok(pool)
}

/// Get the global SignerPool (call init_signer_pool_from_env() first in your bootstrap).
pub fn signer_pool() -> &'static SignerPool {
    SIGNER_POOL
        .get()
        .expect("signer pool is not initialized; call init_signer_pool_from_env() first")
}

#[derive(Debug, Serialize, Deserialize)]
struct SignersFile(HashMap<String, String>);

impl SignerPool {
    /// Save current signers to a JSON file. WARNING: stores raw private keys; for local development only.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let map = self
            .keys
            .read()
            .expect("SignerPool RwLock poisoned while saving")
            .clone();
        let data = serde_json::to_vec_pretty(&SignersFile(map))?;
        if let Some(parent) = path.as_ref().parent() {
            if !parent.as_os_str().is_empty() {
                let _ = fs::create_dir_all(parent);
            }
        }
        fs::write(path, data)?;
        Ok(())
    }

    /// Load signers from a JSON file; merges into current set (file entries override existing).
    pub fn load_from_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let p = path.as_ref();
        if !p.exists() {
            return Ok(());
        }
        let bytes = fs::read(p)?;
        let SignersFile(map): SignersFile = serde_json::from_slice(&bytes)?;
        for (name, pk) in map.into_iter() {
            let _ = PrivateKeySigner::from_str(&pk)?;
            let mut w = self
                .keys
                .write()
                .expect("SignerPool RwLock poisoned while loading");
            w.insert(name, pk);
        }
        Ok(())
    }
}

/// Resolve default signers store path: env SIGNERS_FILE or ".signers.json" in current dir.
pub fn default_signers_store_path() -> PathBuf {
    if let Ok(p) = std::env::var("SIGNERS_FILE") {
        return PathBuf::from(p);
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".signers.json")
}
