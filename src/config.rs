use std::sync::OnceLock;
use std::{env, str::FromStr};

use alloy::primitives::Address;
use dotenv::dotenv;

#[derive(Clone, Debug)]
pub struct FeatureFlags {
    pub print_raw_logs: bool,
    pub print_unknown: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AddressMap {
    pub did_registry: Address,
    pub show_manager: Address,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub ws_rpc_url: String,
    pub database_url: String,
    pub flags: FeatureFlags,
    pub addresses: AddressMap,
}

impl Config {
    pub fn load() -> eyre::Result<Self> {
        // dotenv() is called in main; we don't call here to keep side-effects explicit
        let ws_rpc_url = env::var("WS_RPC_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:8545".to_string());
        if !(ws_rpc_url.starts_with("ws://")
            || ws_rpc_url.starts_with("wss://"))
        {
            eyre::bail!(
                "WS_RPC_URL must start with ws:// or wss://, got: {}",
                ws_rpc_url
            );
        }
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| eyre::eyre!("DATABASE_URL is required"))?;

        let did_s = env::var("DID_REGISTRY_ADDRESS")
            .map_err(|_| eyre::eyre!("DID_REGISTRY_ADDRESS is required"))?;
        let show_s = env::var("SHOW_MANAGER_ADDRESS")
            .map_err(|_| eyre::eyre!("SHOW_MANAGER_ADDRESS is required"))?;
        for (name, val) in [
            ("DID_REGISTRY_ADDRESS", &did_s),
            ("SHOW_MANAGER_ADDRESS", &show_s),
        ] {
            if !val.starts_with("0x") {
                eyre::bail!("{} must start with 0x, got: {}", name, val);
            }
            if val.len() != 42 {
                eyre::bail!(
                    "{} must be a 20-byte hex address (42 chars with 0x), got len {}: {}",
                    name,
                    val.len(),
                    val
                );
            }
        }
        let did = Address::from_str(&did_s)
            .map_err(|e| eyre::eyre!("Invalid DID_REGISTRY_ADDRESS: {}", e))?;
        let show = Address::from_str(&show_s)
            .map_err(|e| eyre::eyre!("Invalid SHOW_MANAGER_ADDRESS: {}", e))?;

        let flags = FeatureFlags {
            print_raw_logs: env::var("PRINT_RAW_LOGS").ok().as_deref()
                == Some("1"),
            print_unknown: env::var("PRINT_UNKNOWN_LOGS").ok().as_deref()
                == Some("1"),
        };

        let addresses = AddressMap {
            did_registry: did,
            show_manager: show,
        };

        Ok(Self {
            ws_rpc_url,
            database_url,
            flags,
            addresses,
        })
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

/// 初始化全局配置（从 .env 环境变量）。应在程序启动时调用一次。
pub fn init_from_env() -> eyre::Result<&'static Config> {
    dotenv().ok();
    let cfg = Config::load()?;
    let _ = CONFIG.set(cfg);
    Ok(CONFIG.get().expect("config must be initialized"))
}

/// 获取全局配置的引用；在 init_from_env() 之后使用。
pub fn get() -> &'static Config {
    CONFIG
        .get()
        .expect("config is not initialized; call init_from_env() first")
}

/// 使用外部提供的 Config 初始化（测试或自定义环境）。
pub fn init_with(cfg: Config) -> &'static Config {
    let _ = CONFIG.set(cfg);
    CONFIG.get().expect("config must be initialized")
}
