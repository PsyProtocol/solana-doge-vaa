use crate::{
    dogecoin::{
        address::BTCAddress160,
        constants::DogeNetworkConfig,
        hash::QHash256,
        transaction::{BTCTransaction, BTCTransactionWithStatus, TXStatus},
    },
    psy_doge_link::{
        data::{ElectrsTxStatus, ElectrsUTXO, FeeEstimateMap, BTCUTXO}, traits::PsyBitcoinAPIAsync,
    },
    tx_store::traits::DogecoinRPCProviderAsync,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

fn trim_trailing_slash(s: &str) -> &str {
    s.strip_suffix('/').unwrap_or(s)
}


impl From<ElectrsTxStatus> for TXStatus {
    fn from(status: ElectrsTxStatus) -> Self {
        Self {
            confirmed: status.confirmed,
            block_height: status.block_height,
            block_hash: status.block_hash,
            block_time: status.block_time,
        }
    }
}
#[derive(Clone)]
pub struct DogeLinkElectrsRPCAsync<N: DogeNetworkConfig> {
    base_url: String,
    client: Client,
    _network: PhantomData<N>,
}

impl<N: DogeNetworkConfig> DogeLinkElectrsRPCAsync<N> {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: trim_trailing_slash(base_url).to_string(),
            client: Client::new(),
            _network: PhantomData,
        }
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let res = self.client.get(&url).send().await?;
        let status = res.status();
        if status.is_success() {
            let json_body = res.json::<T>().await?;
            Ok(json_body)
        } else {
            let error_body = res.text().await?;
            anyhow::bail!("API Error ({} @ {}): {}", status, url, error_body);
        }
    }

    async fn get_text(&self, path: &str) -> anyhow::Result<String> {
        let url = format!("{}{}", self.base_url, path);
        let res = self.client.get(&url).send().await?;
        let status = res.status();
        if status.is_success() {
            let text_body = res.text().await?;
            Ok(text_body)
        } else {
            let error_body = res.text().await?;
            anyhow::bail!("API Error ({} @ {}): {}", status, url, error_body);
        }
    }

    async fn post_text(&self, path: &str, body: String) -> anyhow::Result<String> {
        let url = format!("{}{}", self.base_url, path);
        let res = self
            .client
            .post(&url)
            .header("Content-Type", "text/plain")
            .body(body)
            .send()
            .await?;
        let status = res.status();
        if status.is_success() {
            let text_body = res.text().await?;
            Ok(text_body)
        } else {
            let error_body = res.text().await?;
            anyhow::bail!("API Error ({} @ {}): {}", status, url, error_body);
        }
    }

    pub async fn get_transaction_status(
        &self,
        txid: &QHash256,
    ) -> anyhow::Result<ElectrsTxStatus> {
        self.get_json(&format!("/tx/{}/status", hex::encode(txid)))
            .await
    }

    async fn get_transaction_inner(&self, txid: QHash256) -> anyhow::Result<BTCTransaction> {
        let txid_hex = hex::encode(txid);
        let path = format!("/tx/{}/hex", txid_hex);
        let raw_tx_hex = self.get_text(&path).await?;
        Ok(hex::decode(raw_tx_hex.trim())?)
            .and_then(|raw_tx| BTCTransaction::from_bytes(&raw_tx))
    }


    pub async fn get_transaction_with_status(
        &self,
        txid: &QHash256,
    ) -> anyhow::Result<BTCTransactionWithStatus> {
        let transaction = self.get_transaction_inner(*txid).await?;
        let status = self.get_transaction_status(txid).await?;

        let status: TXStatus = status.into();

        Ok(BTCTransactionWithStatus {
            transaction,
            status,
        })
    }

}

#[async_trait]
impl<N: DogeNetworkConfig + Send + Sync> PsyBitcoinAPIAsync for DogeLinkElectrsRPCAsync<N> {
    async fn get_utxos(&self, address: BTCAddress160) -> anyhow::Result<Vec<BTCUTXO>> {
        let address_str = address.to_address_string::<N>();
        let path = format!("/address/{}/utxo", address_str);
        let electrs_utxos: Vec<ElectrsUTXO> = self.get_json(&path).await?;
        Ok(electrs_utxos.into_iter().map(Into::into).collect())
    }

    async fn estimate_fee_rate(&self, n_blocks: u32) -> anyhow::Result<u64> {
        let fee_map: FeeEstimateMap = self.get_json("/fee-estimates").await?;
        let fee_rate_sats_per_vb = fee_map
            .0
            .get(&n_blocks.to_string())
            .or_else(|| fee_map.0.get("25")) // Fallback to 25 blocks as in TS
            .ok_or_else(|| anyhow::anyhow!("Fee estimate not available for {} blocks", n_blocks))?;

        if *fee_rate_sats_per_vb < 0.0 {
            anyhow::bail!("Negative fee rate received");
        }
        Ok(*fee_rate_sats_per_vb as u64)
    }

    async fn get_transaction(&self, txid: QHash256) -> anyhow::Result<BTCTransaction> {
        let raw_tx = self.get_raw_transaction_by_txid(&txid).await?;
        BTCTransaction::from_bytes(&raw_tx)
    }

    async fn send_transaction(&self, tx: &BTCTransaction) -> anyhow::Result<QHash256> {
        self.submit_raw_transaction(&tx.to_bytes()).await
    }
}

#[async_trait]
impl<N: DogeNetworkConfig + Send + Sync> DogecoinRPCProviderAsync for DogeLinkElectrsRPCAsync<N> {
    async fn get_raw_transaction_by_txid(&self, txid: &QHash256) -> anyhow::Result<Vec<u8>> {
        let txid_hex = hex::encode(txid);
        let path = format!("/tx/{}/hex", txid_hex);
        let raw_tx_hex = self.get_text(&path).await?;
        Ok(hex::decode(raw_tx_hex.trim())?)
    }

    async fn submit_raw_transaction(&self, raw_tx: &[u8]) -> anyhow::Result<QHash256> {
        let tx_hex = hex::encode(raw_tx);
        let returned_txid_hex = self.post_text("/tx", tx_hex).await?;
        let mut txid_bytes = [0u8; 32];
        hex::decode_to_slice(returned_txid_hex.trim(), &mut txid_bytes)?;
        Ok(txid_bytes)
    }
}