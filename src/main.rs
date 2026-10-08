mod inspector;
mod agent;
mod rpc_db;

use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::types::ErrorObjectOwned;
use revm::{
    db::CacheDB,
    primitives::{Address, TxKind, U256, Bytes, BlockEnv, CfgEnv, TxEnv, Env, ExecutionResult},
    Evm,
    inspector_handle_register,
};
use reqwest::Client;
use serde_json::{json, Value};
use std::str::FromStr;
use std::error::Error;
use std::env;

#[rpc(server)]
pub trait LocalGatekeeper {
    #[method(name = "eth_sendTransaction")]
    async fn send_transaction(&self, tx: serde_json::Value) -> Result<String, ErrorObjectOwned>;

    #[method(name = "eth_blockNumber")]
    async fn block_number(&self) -> Result<String, ErrorObjectOwned>;

    #[method(name = "eth_getBalance")]
    async fn get_balance(&self, address: String, block: String) -> Result<String, ErrorObjectOwned>;
}

#[derive(Debug)]
struct SimError(String);

impl std::fmt::Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for SimError {}

fn simulate_transaction(rpc_url: String, tx_env: TxEnv, inspector: &mut inspector::RiskInspector) -> Result<(inspector::EVMStructuralAnalysis, ExecutionResult), Box<dyn Error + Send + Sync>> {
    let db = rpc_db::RpcCacheDB::new(rpc_url);
    let mut cached_db = CacheDB::new(db);

    let block_env = BlockEnv {
        number: U256::from(20_000_000),
        timestamp: U256::from(1700000000),
        gas_limit: U256::from(30_000_000),
        ..Default::default()
    };

    let mut cfg_env = CfgEnv::default();
    cfg_env.chain_id = 1;

    let env = Box::new(Env {
        cfg: cfg_env,
        block: block_env,
        tx: tx_env,
    });

    let mut evm = Evm::builder()
        .with_db(cached_db)
        .with_external_context(inspector)
        .append_handler_register(inspector_handle_register)
        .with_env(env)
        .build();

    let result = evm.transact().map_err(|e| SimError(format!("{}", e)))?;

    // Capture post-execution state using the evm context
    let context = &mut evm.context.evm;
    let inspector = &mut evm.context.external;
    inspector.capture_post_state(context);
    inspector.detect_delegations(context);

    Ok((inspector.analysis.clone(), result.result))
}

pub struct GatekeeperServer {
    rpc_url: String,
    http_client: Client,
}

impl GatekeeperServer {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_url,
            http_client: Client::new(),
        }
    }

    async fn fetch_rpc_value(&self, method: &str, params: Vec<Value>) -> Result<Value, String> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let res = self.http_client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())?;

        Ok(res["result"].clone())
    }

    /// Parse an `eth_sendTransaction`-style JSON value, run the revm
    /// simulation with the RiskInspector, and consult the local LLM agent.
    /// Shared by the interactive proxy path and `--ci` mode.
    pub async fn simulate_from_value(&self, tx: serde_json::Value) -> Result<(inspector::EVMStructuralAnalysis, ExecutionResult, String), ErrorObjectOwned> {
        let from = tx.get("from").and_then(|v| v.as_str()).unwrap_or("");
        let to = tx.get("to").and_then(|v| v.as_str()).unwrap_or("");
        let value = tx.get("value").and_then(|v| v.as_str()).unwrap_or("0x0");
        let data = tx.get("data").and_then(|v| v.as_str()).unwrap_or("0x");
        let gas = tx.get("gas").and_then(|v| v.as_str()).unwrap_or("0x5208");
        let gas_price = tx.get("gasPrice").and_then(|v| v.as_str()).unwrap_or("0x3b9aca00");
        let nonce = tx.get("nonce").and_then(|v| v.as_str()).unwrap_or("0x0");

        let from_addr = Address::from_str(from)
            .unwrap_or_else(|_| Address::from_str("0x1111111111111111111111111111111111111111").unwrap());

        let to_addr = if to.is_empty() {
            TxKind::Create
        } else {
            TxKind::Call(Address::from_str(to).unwrap_or(Address::ZERO))
        };

        let value_u256 = U256::from_str_radix(value.trim_start_matches("0x"), 16).unwrap_or(U256::ZERO);
        let data_bytes = Bytes::from_str(data).unwrap_or_default();
        let gas_u64 = u64::from_str_radix(gas.trim_start_matches("0x"), 16).unwrap_or(21000);
        let gas_price_u256 = U256::from_str_radix(gas_price.trim_start_matches("0x"), 16).unwrap_or(U256::from(1_000_000_000));
        let nonce_u64 = u64::from_str_radix(nonce.trim_start_matches("0x"), 16).unwrap_or(0);

        let tx_env = TxEnv {
            caller: from_addr,
            transact_to: to_addr,
            value: value_u256,
            data: data_bytes,
            gas_limit: gas_u64,
            gas_price: gas_price_u256,
            nonce: Some(nonce_u64),
            chain_id: Some(1),
            ..Default::default()
        };

        let rpc_url = self.rpc_url.clone();
        let mut inspector = inspector::RiskInspector::default();

        let (analysis, exec_result) = tokio::task::spawn_blocking(move || {
            simulate_transaction(rpc_url, tx_env, &mut inspector)
        }).await
        .map_err(|e| ErrorObjectOwned::owned(-32603, format!("Task join error: {}", e), None::<()>))?
        .map_err(|e| ErrorObjectOwned::owned(-32603, format!("Simulation failed: {}", e), None::<()>))?;

        let summary = serde_json::to_string_pretty(&analysis).unwrap_or_else(|_| "{}".to_string());
        let advice = agent::query_local_agent(&summary).await.unwrap_or_else(|_| "Agent offline".to_string());

        Ok((analysis, exec_result, advice))
    }
}

#[async_trait]
impl LocalGatekeeperServer for GatekeeperServer {
    async fn send_transaction(&self, tx: serde_json::Value) -> Result<String, ErrorObjectOwned> {
        println!("\n[*] Intercepted Transaction Request");

        let (analysis, exec_result, advice) = self.simulate_from_value(tx).await?;

        let summary = serde_json::to_string_pretty(&analysis).unwrap_or_else(|_| "{}".to_string());
        println!("[*] Structural Analysis Extracted:\n{}", summary);

        println!("\n================ SECURITY AGENT BRIEF ================");
        println!("{}", advice);
        println!("======================================================");

        if exec_result.is_success() {
            println!("\n[+] Simulation succeeded");
        } else {
            println!("\n[-] Simulation reverted/failed");
        }

        println!("\nProceed with broadcast to network? [y/N]: ");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);

        if input.trim().eq_ignore_ascii_case("y") {
            Ok(format!("0x{:x}", exec_result.gas_used()))
        } else {
            Err(ErrorObjectOwned::owned(-32603, "Transaction aborted by user.", None::<()>))
        }
    }

    async fn block_number(&self) -> Result<String, ErrorObjectOwned> {
        match self.fetch_rpc_value("eth_blockNumber", vec![]).await {
            Ok(val) => Ok(val.as_str().unwrap_or("0x0").to_string()),
            Err(e) => Err(ErrorObjectOwned::owned(-32603, e, None::<()>)),
        }
    }

    async fn get_balance(&self, address: String, _block: String) -> Result<String, ErrorObjectOwned> {
        match self.fetch_rpc_value("eth_getBalance", vec![json!(address), json!("latest")]).await {
            Ok(val) => Ok(val.as_str().unwrap_or("0x0").to_string()),
            Err(e) => Err(ErrorObjectOwned::owned(-32603, e, None::<()>)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "https://eth.llamarpc.com".to_string());
    let args: Vec<String> = env::args().collect();

    // CI mode: read a single tx JSON from stdin, simulate, print structured
    // JSON to stdout, exit 1 when BLOCK heuristics trigger. Never prompts.
    // Example: type examples/demo_txs.json | pretxsim_poc --ci
    if args.iter().any(|a| a == "--ci") {
        let mut input = String::new();
        use std::io::Read;
        std::io::stdin().read_to_string(&mut input).unwrap_or_default();
        let trimmed = input.trim();
        let tx: serde_json::Value = if trimmed.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(trimmed).unwrap_or_else(|_| serde_json::json!({}))
        };
        let server = GatekeeperServer::new(rpc_url);
        match server.simulate_from_value(tx).await {
            Ok((analysis, exec_result, advice)) => {
                let out = serde_json::json!({
                    "analysis": analysis,
                    "execution_success": exec_result.is_success(),
                    "advice": advice,
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
                let is_blocked = analysis.delegatecalls_detected > 0
                    || analysis.selfdestruct_triggered
                    || !analysis.delegation_changes.is_empty()
                    || analysis.storage_slots_written > 10;
                if is_blocked {
                    std::process::exit(1);
                }
                return Ok(());
            }
            Err(e) => {
                let out = serde_json::json!({ "error": e.message(), "code": e.code() });
                eprintln!("{}", serde_json::to_string(&out).unwrap());
                std::process::exit(1);
            }
        }
    }

    println!("======================================================");
    println!("  PreTxSim Micro-Agent Proxy Live on 127.0.0.1:8545");
    println!("  Upstream RPC: {}", rpc_url);
    println!("  RAM Usage target: < 800 MB");
    println!("======================================================");

    let server = ServerBuilder::default().build("127.0.0.1:8545").await?;
    let handle = server.start(GatekeeperServer::new(rpc_url).into_rpc());

    handle.stopped().await;
    Ok(())
}
