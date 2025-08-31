use bevy::prelude::*;
use dotenv::dotenv;
use ethers::prelude::{Provider, Http, SignerMiddleware, LocalWallet, abigen, Middleware};
use ethers::signers::Signer;
use eyre::Result;
use std::{str::FromStr, sync::Arc, fs};
use ethers::types::{Address, U256};
use serde::Deserialize;
use toml;

#[derive(Debug, Deserialize)]
struct StylusConfig {
    contract: ContractConfig,
    deployment: DeploymentConfig,
    functions: FunctionsConfig,
}

#[derive(Debug, Deserialize)]
struct ContractConfig {
    address: String,
    network: String,
    rpc_url: String,
}

#[derive(Debug, Deserialize)]
struct DeploymentConfig {
    tx_hash: String,
    activation_tx_hash: String,
    contract_size: String,
    wasm_size: String,
    wasm_data_fee: String,
}

#[derive(Debug, Deserialize)]
struct FunctionsConfig {
    signatures: Vec<String>,
}

// Generate the contract bindings
abigen!(
    BlockchainContract,
    r#"[
        function getSwordCounts() external view returns (uint256, uint256, uint256)
        function incrementSword(uint256 color) external
    ]"#
);

#[derive(Resource, Clone)]
pub struct StylusClient {
    pub contract_client: Option<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    pub contract_address: Option<Address>,
    pub contract: Option<BlockchainContract<SignerMiddleware<Provider<Http>, LocalWallet>>>,
}

impl StylusClient {
    /// Convert a u8 to U256 for blockchain operations
    pub fn u8_to_u256(&self, value: u8) -> U256 {
        U256::from(value)
    }

    /// Convert a u64 to U256 for blockchain operations
    pub fn u64_to_u256(&self, value: u64) -> U256 {
        U256::from(value)
    }

    /// Convert a u32 to U256 for blockchain operations
    pub fn u32_to_u256(&self, value: u32) -> U256 {
        U256::from(value)
    }

    /// Convert a u16 to U256 for blockchain operations
    pub fn u16_to_u256(&self, value: u16) -> U256 {
        U256::from(value)
    }

    /// Convert a usize to U256 for blockchain operations
    pub fn usize_to_u256(&self, value: usize) -> U256 {
        U256::from(value)
    }

    /// Get sword counts from the blockchain
    pub fn get_sword_counts(&self) -> Result<(u64, u64, u64)> {
        if let Some(contract) = &self.contract {
            let runtime = tokio::runtime::Runtime::new()?;
            let result = runtime.block_on(contract.get_sword_counts().call())?;
            Ok((
                result.0.as_u64(),
                result.1.as_u64(),
                result.2.as_u64(),
            ))
        } else {
            Err(eyre::eyre!("Contract not initialized"))
        }
    }

    /// Increment sword count on the blockchain
    pub fn increment_sword(&self, color: u8) -> Result<()> {
        if let Some(contract) = &self.contract {
            let runtime = tokio::runtime::Runtime::new()?;
            let _ = runtime.block_on(contract.increment_sword(self.u8_to_u256(color)).send())?;
            Ok(())
        } else {
            Err(eyre::eyre!("Contract not initialized"))
        }
    }

    /// Increment sword count on the blockchain asynchronously (spawns a thread)
    pub fn increment_sword_async(&self, color: u8) {
        if let Some(contract) = &self.contract {
            let contract = contract.clone();
            let color_u256 = self.u8_to_u256(color);
            std::thread::spawn(move || {
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    let _ = contract.increment_sword(color_u256).send().await;
                });
            });
        }
    }
}

pub struct StylusPlugin;

impl Plugin for StylusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_stylus);
    }
}

pub fn init_stylus(mut commands: Commands) {
    let stylus_client = std::thread::spawn(|| {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                init_stylus_client().await
            })
    })
    .join()
    .unwrap();

    match stylus_client {
        Ok(client) => {
            println!("✅ Stylus client initialized successfully");
            commands.insert_resource(client);
        }
        Err(e) => {
            println!("❌ Failed to initialize Stylus client: {:?}", e);
            commands.insert_resource(StylusClient {
                contract_client: None,
                contract_address: None,
                contract: None,
            });
        }
    }
}

async fn init_stylus_client() -> Result<StylusClient> {
    dotenv().ok();

    let mut client = StylusClient {
        contract_client: None,
        contract_address: None,
        contract: None,
    };

    // Read Stylus.toml configuration
    let config_content = fs::read_to_string("Stylus.toml")
        .map_err(|e| eyre::eyre!("Failed to read Stylus.toml: {}", e))?;
    
    let config: StylusConfig = toml::from_str(&config_content)
        .map_err(|e| eyre::eyre!("Failed to parse Stylus.toml: {}", e))?;

    println!("📋 Loaded Stylus configuration:");
    println!("  - Contract Address: {}", config.contract.address);
    println!("  - Network: {}", config.contract.network);
    println!("  - RPC URL: {}", config.contract.rpc_url);
    println!("  - Functions: {} signatures", config.functions.signatures.len());

    // Get private key from environment or use default
    let private_key = std::env::var("PRIVATE_KEY")
        .unwrap_or_else(|_| "0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659".to_string());

    println!("🔑 Using private key: {}", if private_key.len() > 10 { 
        format!("{}...{}", &private_key[..10], &private_key[private_key.len()-10..]) 
    } else { 
        private_key.clone() 
    });

    // Create provider and wallet
    let provider = Provider::<Http>::try_from(&config.contract.rpc_url)?;
    let wallet = LocalWallet::from_str(&private_key)?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let client_arc = Arc::new(SignerMiddleware::new(
        provider,
        wallet.with_chain_id(chain_id),
    ));

    let contract_address: Address = config.contract.address.parse()?;
    let contract = BlockchainContract::new(contract_address, client_arc.clone());

    client.contract_client = Some(client_arc);
    client.contract_address = Some(contract_address);
    client.contract = Some(contract);

    println!("✅ Stylus client initialized successfully!");

    Ok(client)
}

// Re-export the contract type for convenience
pub use BlockchainContract;