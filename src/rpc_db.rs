use revm::{
    primitives::{Address, U256, B256, Bytecode, AccountInfo},
    Database, DatabaseRef,
};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::RwLock;
use std::str::FromStr;

// Standard JSON-RPC methods understood by anvil / any Ethereum node.
// (The previous implementation used non-standard `eth_getAccount` and
// `eth_getCodeByHash`, which no public provider or anvil answers.)
const METHOD_GET_BALANCE: &str = "eth_getBalance";
const METHOD_GET_NONCE: &str = "eth_getTransactionCount";
const METHOD_GET_CODE: &str = "eth_getCode";
const METHOD_GET_STORAGE: &str = "eth_getStorageAt";
const METHOD_GET_BLOCK: &str = "eth_getBlockByNumber";

#[derive(Debug)]
pub struct RpcCacheDB {
    client: Client,
    rpc_url: String,
    block_number: u64,
    // Interior mutability: DatabaseRef::basic_ref takes &self, but the
    // caches must be populated on read. RwLock (not RefCell) because the DB
    // is used inside spawn_blocking and must be Send + Sync.
    cache: RwLock<HashMap<Address, Option<AccountInfo>>>,
    code_cache: RwLock<HashMap<B256, Bytecode>>,
    storage_cache: RwLock<HashMap<(Address, U256), U256>>,
}

impl RpcCacheDB {
    pub fn new(rpc_url: String) -> Self {
        Self {
            client: Client::new(),
            rpc_url,
            block_number: 0,
            cache: RwLock::new(HashMap::new()),
            code_cache: RwLock::new(HashMap::new()),
            storage_cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_block_number(&mut self, block_number: u64) {
        self.block_number = block_number;
    }

    /// Block tag for queries: "latest" unless a specific block was set.
    /// (block_number defaults to 0, which previously pinned every query to
    /// genesis and hid any state injected via anvil_setCode.)
    fn block_tag(&self) -> String {
        if self.block_number == 0 {
            "latest".to_string()
        } else {
            format!("0x{:x}", self.block_number)
        }
    }

    async fn fetch_rpc_value(&self, method: &str, params: Vec<Value>) -> Result<Value, String> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        Ok(self.post_json(payload).await?.remove(0).get("result").cloned().unwrap_or(Value::Null))
    }

    async fn post_json(&self, payload: Value) -> Result<Vec<Value>, String> {
        let res = self.client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("RPC returned status {}", res.status()));
        }

        let body: Value = res.json().await.map_err(|e| e.to_string())?;

        // Batch responses come back as an array; single responses as an object.
        if body.is_array() {
            Ok(body.as_array().cloned().unwrap_or_default())
        } else {
            Ok(vec![body])
        }
    }

    fn block_on<F>(&self, f: F) -> F::Output
    where
        F: std::future::Future + Send,
        F::Output: Send,
    {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(f)
        })
    }
}

impl DatabaseRef for RpcCacheDB {
    type Error = String;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        if let Some(cached) = self.cache.read().unwrap().get(&address) {
            return Ok(cached.clone());
        }

        let addr_str = format!("{:#x}", address);
        let block_tag = self.block_tag();

        // Standard batch: balance + nonce + code in one round trip.
        let batch = json!([
            {"jsonrpc": "2.0", "method": METHOD_GET_BALANCE, "params": [addr_str, block_tag], "id": 1},
            {"jsonrpc": "2.0", "method": METHOD_GET_NONCE, "params": [addr_str, block_tag], "id": 2},
            {"jsonrpc": "2.0", "method": METHOD_GET_CODE, "params": [addr_str, block_tag], "id": 3}
        ]);
        let responses = self.block_on(self.post_json(batch))?;

        let balance_raw = responses.get(0)
            .ok_or_else(|| "batch response missing balance entry".to_string())?
            .get("result").cloned().unwrap_or(Value::Null);
        let nonce_raw = responses.get(1)
            .ok_or_else(|| "batch response missing nonce entry".to_string())?
            .get("result").cloned().unwrap_or(Value::Null);
        let code_raw = responses.get(2)
            .ok_or_else(|| "batch response missing code entry".to_string())?
            .get("result").cloned().unwrap_or(Value::Null);

        let balance = U256::from_str_radix(
            balance_raw.as_str().unwrap_or("0x0").trim_start_matches("0x"),
            16
        ).unwrap_or_default();

        let nonce = u64::from_str_radix(
            nonce_raw.as_str().unwrap_or("0x0").trim_start_matches("0x"),
            16
        ).unwrap_or_default();

        let code_bytes = hex::decode(
            code_raw.as_str().unwrap_or("0x").trim_start_matches("0x")
        ).unwrap_or_default();
        let bytecode = Bytecode::new_raw(code_bytes.into());
        let code_hash = bytecode.hash_slow();

        let account_info = AccountInfo::new(balance, nonce, code_hash, bytecode.clone());

        // Cache code by hash so revm's later code_by_hash lookups resolve
        // locally (standard RPC has no eth_getCodeByHash).
        self.code_cache.write().unwrap().insert(code_hash, bytecode.clone());
        self.cache.write().unwrap().insert(address, Some(account_info.clone()));

        Ok(Some(account_info))
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        if let Some(cached) = self.code_cache.read().unwrap().get(&code_hash) {
            return Ok(cached.clone());
        }

        // Unreachable in practice: every account loaded via basic_ref
        // registers its code here first. Return empty rather than hitting a
        // non-standard method.
        Ok(Bytecode::default())
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        let key = (address, index);
        if let Some(cached) = self.storage_cache.read().unwrap().get(&key) {
            return Ok(*cached);
        }

        let block_tag = self.block_tag();
        let result = self.block_on(self.fetch_rpc_value(METHOD_GET_STORAGE, vec![
            json!(format!("{:#x}", address)),
            json!(format!("0x{:x}", index)),
            json!(block_tag),
        ]))?;

        let value = U256::from_str_radix(
            result.as_str().unwrap_or("0x0").trim_start_matches("0x"),
            16
        ).unwrap_or_default();

        Ok(value)
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        let result = self.block_on(self.fetch_rpc_value(METHOD_GET_BLOCK, vec![
            json!(format!("0x{:x}", number)),
            json!(false),
        ]))?;

        let hash_str = result["hash"].as_str().unwrap_or("");
        let hash = B256::from_str(hash_str).unwrap_or_default();
        Ok(hash)
    }
}

impl Database for RpcCacheDB {
    type Error = String;

    #[inline]
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        <Self as DatabaseRef>::basic_ref(self, address)
    }

    #[inline]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        <Self as DatabaseRef>::code_by_hash_ref(self, code_hash)
    }

    #[inline]
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        <Self as DatabaseRef>::storage_ref(self, address, index)
    }

    #[inline]
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        <Self as DatabaseRef>::block_hash_ref(self, number)
    }
}