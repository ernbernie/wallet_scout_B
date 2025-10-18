use crate::errors::{ScoutError, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

pub struct Rpc {
    url: String,
    http: Client,
}

impl Rpc {
    pub fn new(url: &str) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(ScoutError::Network)?;
        
        Ok(Self {
            url: url.to_string(),
            http,
        })
    }

    /// Make JSON-RPC call with exponential backoff retry
    async fn call<T: for<'de> Deserialize<'de>>(
        &self, 
        method: &str, 
        params: serde_json::Value
    ) -> Result<T> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let mut retries = 3;
        let mut delay = Duration::from_millis(100);

        loop {
            match self.try_call(&payload).await {
                Ok(result) => return Ok(result),
                Err(ScoutError::RateLimited { retry_after }) => {
                    if retries == 0 {
                        return Err(ScoutError::RateLimited { retry_after });
                    }
                    retries -= 1;
                    sleep(Duration::from_secs(retry_after)).await;
                }
                Err(_e) if retries > 0 => {
                    retries -= 1;
                    sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn try_call<T: for<'de> Deserialize<'de>>(
        &self,
        payload: &serde_json::Value,
    ) -> Result<T> {
        let resp = self.http
            .post(&self.url)
            .json(payload)
            .send()
            .await
            .map_err(ScoutError::Network)?;

        // Check for rate limiting
        if let Some(retry_after) = resp.headers().get("retry-after") {
            if let Ok(seconds) = retry_after.to_str().unwrap_or("0").parse::<u64>() {
                return Err(ScoutError::RateLimited { retry_after: seconds });
            }
        }

        let status = resp.status();
        if !status.is_success() {
            return Err(ScoutError::Rpc {
                code: status.as_u16() as i32,
                message: format!("HTTP {}", status),
            });
        }

        let v: serde_json::Value = resp.json().await.map_err(ScoutError::Network)?;
        
        // Check for RPC error in response
        if let Some(error) = v.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) as i32;
            let message = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown RPC error")
                .to_string();
            
            return Err(ScoutError::Rpc { code, message });
        }

        serde_json::from_value(v["result"].clone())
            .map_err(|e| ScoutError::Rpc {
                code: -1,
                message: format!("Failed to deserialize response: {}", e),
            })
    }

    /// Get SOL balance for an address
    pub async fn get_balance(&self, addr: &str) -> Result<f64> {
        #[derive(Deserialize)]
        struct BalanceResponse {
            value: u64,
        }

        let r: BalanceResponse = self.call(
            "getBalance",
            json!([addr, {"commitment": "confirmed"}])
        ).await?;

        // Convert lamports to SOL (1 SOL = 1,000,000,000 lamports)
        Ok(r.value as f64 / 1_000_000_000.0)
    }

    /// Get token accounts for an owner with base64 encoding
    pub async fn get_token_accounts_by_owner_base64(&self, addr: &str) -> Result<Vec<TokenUi>> {
        #[derive(Deserialize)]
        struct AccountData {
            data: (String, String), // (base64_data, encoding)
        }
        
        #[derive(Deserialize)]
        struct Account {
            pubkey: String,
            account: AccountData,
        }
        
        #[derive(Deserialize)]
        struct TokenAccountsResponse {
            value: Vec<Account>,
        }

        // SPL Token program ID
        let program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
        
        let r: TokenAccountsResponse = self.call(
            "getTokenAccountsByOwner",
            json!([
                addr,
                {"programId": program},
                {"encoding": "base64", "commitment": "confirmed"}
            ])
        ).await?;

        Ok(r.value.into_iter().map(|v| TokenUi {
            pubkey: v.pubkey,
            data_base64: v.account.data.0,
        }).collect())
    }

    /// Get multiple accounts in a single batch call
    pub async fn get_multiple_accounts_base64(&self, pubkeys: &[String]) -> Result<Vec<Option<AccountData>>> {
        #[derive(Deserialize)]
        struct RpcAccountData {
            data: (String, String), // (base64_data, encoding)
            executable: bool,
            lamports: u64,
            owner: String,
            rent_epoch: u64,
        }
        
        #[derive(Deserialize)]
        struct MultipleAccountsResponse {
            value: Vec<Option<RpcAccountData>>,
        }

        let r: MultipleAccountsResponse = self.call(
            "getMultipleAccounts",
            json!([
                pubkeys,
                {"encoding": "base64", "commitment": "confirmed"}
            ])
        ).await?;

        Ok(r.value.into_iter().map(|opt| opt.map(|acc| AccountData {
            data: acc.data,
            executable: acc.executable,
            lamports: acc.lamports,
            owner: acc.owner,
            rent_epoch: acc.rent_epoch,
        })).collect())
    }
}

/// Token account data from RPC
#[derive(Debug)]
pub struct TokenUi {
    pub pubkey: String,
    pub data_base64: String,
}

/// Account data from getMultipleAccounts
#[derive(Debug)]
pub struct AccountData {
    pub data: (String, String),
    pub executable: bool,
    pub lamports: u64,
    pub owner: String,
    pub rent_epoch: u64,
}
