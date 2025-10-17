use std::collections::HashMap;

use crate::{dogecoin::{hash::QHash256, transaction::BTCTransaction}, tx_store::traits::{DogecoinRPCProviderAsync, DogecoinRPCProviderSync}};

#[derive(Clone)]
pub struct DogecoinTransactionProviderWithCache<P>{
    rpc_provider: P,
    cache: HashMap<QHash256, BTCTransaction>,
}

impl <P> DogecoinTransactionProviderWithCache<P> {
    pub fn new(rpc_provider: P) -> Self {
        Self {
            rpc_provider,
            cache: HashMap::new(),
        }
    }
}
#[async_trait::async_trait]
impl<P: DogecoinRPCProviderAsync + Sync> DogecoinRPCProviderAsync for DogecoinTransactionProviderWithCache<P> {
    async fn get_raw_transaction_by_txid(&self, txid: &QHash256) -> anyhow::Result<Vec<u8>> {
        if let Some(cached_tx) = self.cache.get(txid) {
            return Ok(cached_tx.to_bytes());
        }
        self.rpc_provider.get_raw_transaction_by_txid(txid).await
    }
    async fn get_transaction_by_txid(&self, txid: &QHash256) -> anyhow::Result<BTCTransaction> {
        if let Some(cached_tx) = self.cache.get(txid) {
            return Ok(cached_tx.clone());
        }
        let tx = self.rpc_provider.get_transaction_by_txid(txid).await?;
        Ok(tx)
    }
    
    async fn submit_raw_transaction(&self, raw_tx: &[u8]) -> anyhow::Result<QHash256> {
        self.rpc_provider.submit_raw_transaction(raw_tx).await
    }
}
impl<P: DogecoinRPCProviderSync> DogecoinRPCProviderSync for DogecoinTransactionProviderWithCache<P> {
    fn get_raw_transaction_by_txid_sync(&self, txid: &QHash256) -> anyhow::Result<Vec<u8>> {
        if let Some(cached_tx) = self.cache.get(txid) {
            return Ok(cached_tx.to_bytes());
        }
        self.rpc_provider.get_raw_transaction_by_txid_sync(txid)
    }
    fn get_transaction_by_txid_sync(&self, txid: &QHash256) -> anyhow::Result<BTCTransaction> {
        if let Some(cached_tx) = self.cache.get(txid) {
            return Ok(cached_tx.clone());
        }
        let tx = self.rpc_provider.get_transaction_by_txid_sync(txid)?;
        Ok(tx)
    }
    
    fn submit_raw_transaction_sync(&self, raw_tx: &[u8]) -> anyhow::Result<QHash256> {
        self.rpc_provider.submit_raw_transaction_sync(raw_tx)
    }
}