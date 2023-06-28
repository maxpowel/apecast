use serde_aux::prelude::deserialize_number_from_string;
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct BalanceResponse {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    status: i32,
    result: String,
    message: String
}

pub struct EtherscanClient {
    token: String
}

impl EtherscanClient {
    pub fn new(token: Option<String>) -> EtherscanClient {
        
        if let Some(token) = token {
            EtherscanClient {
                token
            }
        } else {
            EtherscanClient {
                token: "".to_owned()
            }
        }
        
    }

    pub async fn balance(&self, address: &str) -> Result<f64> {
        let url = format!("https://api.etherscan.io/api?module=account&action=tokenbalance&contractaddress=0x4d224452801aced8b2f0aebe155379bb5d594381&address={}&tag=latest&apikey={}", address, self.token);
        let res: BalanceResponse = reqwest::get(url).await?.json().await?;
        if res.status != 1 {
            Err(anyhow::anyhow!("{}", res.message))
        } else {
            let amount: f64 = (res.result[0..res.result.len() - 15]).parse().unwrap();
            Ok(amount/1000.0)
        }
    }
}