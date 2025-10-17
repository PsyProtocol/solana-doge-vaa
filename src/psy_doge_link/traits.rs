use crate::{dogecoin::{address::BTCAddress160, hash::QHash256, transaction::BTCTransaction}, psy_doge_link::data::{BTCTransactionWithVout, PartialBTCUTXO, BTCUTXO}};
use async_trait::async_trait;


pub trait PsyBitcoinAPISync {
    fn get_funding_transactions(
        &self,
        address: BTCAddress160,
    ) -> anyhow::Result<Vec<BTCTransaction>>;
    fn get_confirmed_funding_transactions_with_vout(
        &self,
        address: BTCAddress160,
    ) -> anyhow::Result<Vec<BTCTransactionWithVout>>;
    fn get_utxos(&self, address: BTCAddress160) -> anyhow::Result<Vec<BTCUTXO>>;
    fn estimate_fee_rate(&self, n_blocks: u32) -> anyhow::Result<u64>;
    fn reset_cached_fee_rate(&mut self, n_blocks: u32) -> anyhow::Result<u64>;
    fn get_cached_fee_rate(&self) -> anyhow::Result<u64>;
    fn get_partial_utxos(&self, address: BTCAddress160) -> anyhow::Result<Vec<PartialBTCUTXO>> {
        Ok(self
            .get_utxos(address)?
            .into_iter()
            .map(|utxo| PartialBTCUTXO {
                txid: utxo.txid,
                vout: utxo.vout,
                value: utxo.value,
            })
            .collect())
    }
    fn get_funding_transactions_with_vout(
        &self,
        address: BTCAddress160,
    ) -> anyhow::Result<Vec<BTCTransactionWithVout>>;
    fn get_transaction(&self, txid: QHash256) -> anyhow::Result<BTCTransaction>;
    fn send_transaction(&self, tx: &BTCTransaction) -> anyhow::Result<QHash256>;
}


#[async_trait]
pub trait PsyBitcoinAPIAsync {
    async fn get_utxos(&self, address: BTCAddress160) -> anyhow::Result<Vec<BTCUTXO>>;
    async fn estimate_fee_rate(&self, n_blocks: u32) -> anyhow::Result<u64>;
    async fn get_transaction(&self, txid: QHash256) -> anyhow::Result<BTCTransaction>;
    async fn send_transaction(&self, tx: &BTCTransaction) -> anyhow::Result<QHash256>;
}