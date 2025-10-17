
use crate::dogecoin::{hash::QHash256, transaction::BTCTransaction};


#[async_trait::async_trait]
pub trait DogecoinRPCProviderAsync {
    // gets a raw transaction from the chain by its txid
    async fn get_raw_transaction_by_txid(&self, txid: &QHash256) -> anyhow::Result<Vec<u8>>;
    
    async fn get_transaction_by_txid(&self, txid: &QHash256) -> anyhow::Result<BTCTransaction> {
        let raw_tx = self.get_raw_transaction_by_txid(txid).await?;
        let (tx, _) = BTCTransaction::from_bytes_offset(&raw_tx, 0)?;
        Ok(tx)
    }
    async fn get_transactions_by_txid(
        &self,
        txids: &[QHash256],
    ) -> anyhow::Result<Vec<BTCTransaction>> {
        let mut results = vec![];
        for txid in txids {
            let tx = self.get_transaction_by_txid(txid).await?;
            results.push(tx);
        }
        Ok(results)
    }
    async fn submit_raw_transaction(&self, raw_tx: &[u8]) -> anyhow::Result<QHash256>;
}

pub trait DogecoinRPCProviderSync {
    fn submit_raw_transaction_sync(&self, raw_tx: &[u8]) -> anyhow::Result<QHash256>;
    fn get_raw_transaction_by_txid_sync(&self, txid: &QHash256) -> anyhow::Result<Vec<u8>>;
    fn get_transaction_by_txid_sync(&self, txid: &QHash256) -> anyhow::Result<BTCTransaction> {
        let raw_tx = self.get_raw_transaction_by_txid_sync(txid)?;
        let (tx, _) = BTCTransaction::from_bytes_offset(&raw_tx, 0)?;
        Ok(tx)
    }
    fn get_transactions_by_txid_sync(
        &self,
        txids: &[QHash256],
    ) -> anyhow::Result<Vec<BTCTransaction>> {
        let mut results = vec![];
        for txid in txids {
            let tx = self.get_transaction_by_txid_sync(txid)?;
            results.push(tx);
        }
        Ok(results)
    }
}
// naive implementation for demonstration purposes
#[cfg(feature = "std")]
impl DogecoinRPCProviderSync for std::collections::HashMap<QHash256, BTCTransaction> {

    fn submit_raw_transaction_sync(&self, _raw_tx: &[u8]) -> anyhow::Result<QHash256> {
        unimplemented!("Submitting transactions is not supported in this cache implementation");
    }
    fn get_raw_transaction_by_txid_sync(&self, txid: &QHash256) -> anyhow::Result<Vec<u8>> {
        let tx = self
            .get(txid)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found in cache"))?;
        Ok(tx.to_bytes())
    }
    fn get_transaction_by_txid_sync(&self, txid: &QHash256) -> anyhow::Result<BTCTransaction> {
        let tx = self
            .get(txid)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found in cache"))?;
        Ok(tx.clone())
    }
}

#[cfg(feature = "std")]
#[async_trait::async_trait]
impl DogecoinRPCProviderAsync for std::collections::HashMap<QHash256, BTCTransaction> {
    async fn submit_raw_transaction(&self, _raw_tx: &[u8]) -> anyhow::Result<QHash256> {
        unimplemented!("Submitting transactions is not supported in this cache implementation");
    }
    async fn get_raw_transaction_by_txid(&self, txid: &QHash256) -> anyhow::Result<Vec<u8>> {
        let tx = self
            .get(txid)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found in cache"))?;
        Ok(tx.to_bytes())
    }
    async fn get_transaction_by_txid(&self, txid: &QHash256) -> anyhow::Result<BTCTransaction> {
        let tx = self
            .get(txid)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found in cache"))?;
        Ok(tx.clone())
    }
    async fn get_transactions_by_txid(
        &self,
        txids: &[QHash256],
    ) -> anyhow::Result<Vec<BTCTransaction>> {
        let mut results = vec![];
        for txid in txids {
            let tx = self.get_transaction_by_txid(txid).await?;
            results.push(tx);
        }
        Ok(results)
    }
}

