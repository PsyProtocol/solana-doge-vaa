use crate::{
    dogecoin::{
        address::BTCAddress160,
        constants::DogeNetworkConfig,
        hash::{DogeHashProvider, QHash256},
        sighash::{SIGHASH_ALL, SigHashPreimage},
        transaction::{
            BTCTransaction, BTCTransactionInput, BTCTransactionInputWithoutScript,
            BTCTransactionOutput,
        },
    },
    tx_store::traits::{DogecoinRPCProviderAsync, DogecoinRPCProviderSync},
    wormhole::script_template::construct_wormhole_vaa_script,
};

// The metadata in the message sent by the contract
#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd, Copy)]
pub struct WormholeBitcoinLikeVAAMetadata {
    pub emitter_chain: u16,
    pub emitter_contract_address: [u8; 32],
    pub sub_address_seed: [u8; 32],
    pub total_output_amount: u64,
    pub max_doge_transaction_fee: u64,
    pub min_doge_transaction_fee: u64,
}

impl WormholeBitcoinLikeVAAMetadata {
    pub fn get_locking_script(&self, guardian_public_key_hash: &[u8; 20]) -> Vec<u8> {
        construct_wormhole_vaa_script(
            self.emitter_chain,
            &self.emitter_contract_address,
            &self.sub_address_seed,
            guardian_public_key_hash,
        )
    }
    pub fn get_p2sh_address<N: DogeNetworkConfig, Hasher: DogeHashProvider>(
        &self,
        guardian_public_key_hash: &[u8; 20],
    ) -> BTCAddress160 {
        BTCAddress160::new_p2sh(Hasher::bitcoin_hash160(
            &self.get_locking_script(guardian_public_key_hash),
        ))
    }
}

#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd)]
pub struct WormholeBitcoinLikeVAAMessage {
    pub metadata: WormholeBitcoinLikeVAAMetadata,
    pub inputs: Vec<BTCTransactionInputWithoutScript>,
    pub outputs: Vec<BTCTransactionOutput>,
}
fn checked_add_sum(list: &[u64]) -> anyhow::Result<u64> {
    let mut total: u64 = 0;
    for item in list.iter() {
        total = total
            .checked_add(*item)
            .ok_or_else(|| anyhow::anyhow!("u64 overflow in addition"))?;
    }
    Ok(total)
}
impl WormholeBitcoinLikeVAAMessage {
    fn validate_and_get_sighashes_with_inputs<Hasher: DogeHashProvider, N: DogeNetworkConfig>(
        &self,
        input_transactions: &[BTCTransaction],
        guardian_public_key_hash: &[u8; 20],
    ) -> anyhow::Result<(Vec<QHash256>, BTCTransaction)> {
        let expected_address = self
            .metadata
            .get_p2sh_address::<N, Hasher>(guardian_public_key_hash);

        let redeem_script = self.metadata.get_locking_script(guardian_public_key_hash);

        let mut total_input_amount: u64 = 0;
        for (input_transaction, input) in input_transactions.iter().zip(self.inputs.iter()) {
            let actual_hash = input_transaction.get_hash::<Hasher>();
            let expected_hash = input.hash;
            if actual_hash != expected_hash {
                anyhow::bail!(
                    "RPC returned the wrong transaction, we requested hash={:?} but got hash={:?}",
                    expected_hash,
                    actual_hash
                );
            }
            if !input_transaction.has_vout_for_address(&expected_address, input.index as usize) {
                anyhow::bail!("input does not spend from the expected wormhole vaa p2sh address");
            }
            total_input_amount = total_input_amount
                .checked_add(input_transaction.outputs[input.index as usize].value)
                .ok_or_else(|| anyhow::anyhow!("u64 overflow in addition"))?;
        }
        let actual_total_output_amount = checked_add_sum(
            &self
                .outputs
                .iter()
                .map(|output| output.value)
                .collect::<Vec<u64>>(),
        )?;
        let expected_total_output_amount = self.metadata.total_output_amount;
        if actual_total_output_amount != expected_total_output_amount {
            anyhow::bail!(
                "total output amount does not match metadata, expected {} but got {}",
                expected_total_output_amount,
                actual_total_output_amount
            );
        }
        let total_fees_paid = total_input_amount
            .checked_sub(actual_total_output_amount)
            .ok_or_else(|| anyhow::anyhow!("u64 underflow in subtraction"))?;
        let max_fee = self.metadata.max_doge_transaction_fee;
        let min_fee = self.metadata.min_doge_transaction_fee;
        if total_fees_paid < min_fee {
            anyhow::bail!(
                "transaction fee paid {} is less than minimum required {}",
                total_fees_paid,
                min_fee
            );
        } else if total_fees_paid > max_fee {
            anyhow::bail!(
                "transaction fee paid {} is more than maximum allowed {}",
                total_fees_paid,
                max_fee
            );
        }
        let base_tx = BTCTransaction {
            version: 2,
            inputs: self
                .inputs
                .iter()
                .map(|x| BTCTransactionInput {
                    hash: x.hash,
                    sequence: x.sequence,
                    script: vec![],
                    index: x.index,
                })
                .collect(),
            outputs: self.outputs.clone(),
            locktime: 0,
        };
        let sighashes = (0..self.inputs.len())
            .map(|i: usize| {
                SigHashPreimage::for_transaction_pre_segwit(
                    &base_tx,
                    i,
                    &redeem_script, // FIXED: Use redeem_script here instead of vaa_address_output_script
                    SIGHASH_ALL,
                )
                .get_hash::<Hasher>()
            })
            .collect();
        Ok((sighashes, base_tx))
    }

    // in a production scenario, you would likely want to use an async RPC provider and cache the transactions
    pub fn validate_and_get_sighashes_sync<
        Hasher: DogeHashProvider,
        N: DogeNetworkConfig,
        RPC: DogecoinRPCProviderSync,
    >(
        &self,
        rpc_provider: &RPC,
        guardian_public_key_hash: &[u8; 20],
    ) -> anyhow::Result<(Vec<QHash256>, BTCTransaction)> {
        let input_txids = self
            .inputs
            .iter()
            .map(|input| input.get_txid())
            .collect::<Vec<QHash256>>();

        let input_transactions = rpc_provider.get_transactions_by_txid_sync(&input_txids)?;
        self.validate_and_get_sighashes_with_inputs::<Hasher, N>(
            &input_transactions,
            guardian_public_key_hash,
        )
    }
    pub async fn validate_and_get_sighashes_async<
        Hasher: DogeHashProvider,
        N: DogeNetworkConfig,
        RPC: DogecoinRPCProviderAsync + Sync,
    >(
        &self,
        rpc_provider: &RPC,
        guardian_public_key_hash: &[u8; 20],
    ) -> anyhow::Result<(Vec<QHash256>, BTCTransaction)> {
        let input_txids = self
            .inputs
            .iter()
            .map(|input| input.get_txid())
            .collect::<Vec<QHash256>>();
        let input_transactions = rpc_provider.get_transactions_by_txid(&input_txids).await?;
        self.validate_and_get_sighashes_with_inputs::<Hasher, N>(
            &input_transactions,
            guardian_public_key_hash,
        )
    }
}
