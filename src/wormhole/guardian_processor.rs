use crate::{
    dogecoin::{
        constants::DogeNetworkConfig,
        hash::{DogeHashProvider, QHash256},
        transaction::{BTCTransaction, encode_binary_stack_item},
    },
    tx_store::traits::{DogecoinRPCProviderAsync, DogecoinRPCProviderSync},
    wormhole::{
        p2sh_vaa::WormholeBitcoinLikeVAAMessage,
        tss_signer::{WormholeTSSSignerAsync, WormholeTSSSignerSync},
    },
};

pub struct WormholeGuardianProcessorAsync<
    RPC: DogecoinRPCProviderAsync,
    Signer: WormholeTSSSignerAsync,
> {
    tss_public_key_hash: [u8; 20],
    rpc_provider: RPC,
    signer: Signer,
}

impl<RPC: DogecoinRPCProviderAsync + Sync, Signer: WormholeTSSSignerAsync>
    WormholeGuardianProcessorAsync<RPC, Signer>
{
    pub fn new_with_tss_public_key_hash(
        rpc_provider: RPC,
        signer: Signer,
        tss_public_key_hash: [u8; 20],
    ) -> Self {
        Self {
            tss_public_key_hash,
            rpc_provider,
            signer,
        }
    }
    pub fn new<Hasher: DogeHashProvider>(rpc_provider: RPC, signer: Signer) -> Self {
        let tss_public_key_hash = Hasher::bitcoin_hash160(&signer.get_tss_public_key().0);

        Self {
            tss_public_key_hash,
            rpc_provider,
            signer,
        }
    }
    pub async fn validate_p2sh_vaa_message_and_sign_async<
        Hasher: DogeHashProvider,
        N: DogeNetworkConfig,
    >(
        &self,
        message: WormholeBitcoinLikeVAAMessage,
    ) -> anyhow::Result<BTCTransaction> {
        let (sighashes, mut spend_transaction) = message
            .validate_and_get_sighashes_async::<Hasher, N, RPC>(
                &self.rpc_provider,
                &self.tss_public_key_hash,
            )
            .await?;
        if sighashes.len() != message.inputs.len() {
            return Err(anyhow::anyhow!("Invalid number of sighashes returned"));
        }
        for (i, sighash) in sighashes.into_iter().enumerate() {
            let signature = self
                .signer
                .sign_message_tss_and_broadcast_to_other_guardians_and_wait_for_signature(sighash)
                .await?;
            let mut input_script = signature.to_btc_script();
            let redeem_script = message
                .metadata
                .get_locking_script(&self.tss_public_key_hash);
            input_script.extend_from_slice(&encode_binary_stack_item(&redeem_script));
            spend_transaction.inputs[i].script = input_script;
        }

        Ok(spend_transaction)
    }
    pub async fn validate_p2sh_vaa_message_sign_and_broadcast_asyncc<
        Hasher: DogeHashProvider,
        N: DogeNetworkConfig,
    >(
        &self,
        message: WormholeBitcoinLikeVAAMessage,
    ) -> anyhow::Result<QHash256> {
        let spend_transaction = self
            .validate_p2sh_vaa_message_and_sign_async::<Hasher, N>(message)
            .await?;
        let raw_tx = spend_transaction.to_bytes();
        let txid = self.rpc_provider.submit_raw_transaction(&raw_tx).await?;
        Ok(txid)
    }
}

pub struct WormholeGuardianProcessorSync<
    RPC: DogecoinRPCProviderSync,
    Signer: WormholeTSSSignerSync,
> {
    tss_public_key_hash: [u8; 20],
    rpc_provider: RPC,
    signer: Signer,
}

impl<RPC: DogecoinRPCProviderSync, Signer: WormholeTSSSignerSync>
    WormholeGuardianProcessorSync<RPC, Signer>
{
    pub fn new_with_tss_public_key_hash(
        rpc_provider: RPC,
        signer: Signer,
        tss_public_key_hash: [u8; 20],
    ) -> Self {
        Self {
            tss_public_key_hash,
            rpc_provider,
            signer,
        }
    }
    pub fn new<Hasher: DogeHashProvider>(rpc_provider: RPC, signer: Signer) -> Self {
        let tss_public_key_hash = Hasher::bitcoin_hash160(&signer.get_tss_public_key().0);

        Self {
            tss_public_key_hash,
            rpc_provider,
            signer,
        }
    }
    pub fn validate_p2sh_vaa_message_and_sign_sync<
        Hasher: DogeHashProvider,
        N: DogeNetworkConfig,
    >(
        &self,
        message: WormholeBitcoinLikeVAAMessage,
    ) -> anyhow::Result<BTCTransaction> {
        let (sighashes, mut spend_transaction) = message
            .validate_and_get_sighashes_sync::<Hasher, N, RPC>(
                &self.rpc_provider,
                &self.tss_public_key_hash,
            )?;
        if sighashes.len() != message.inputs.len() {
            return Err(anyhow::anyhow!("Invalid number of sighashes returned"));
        }
        for (i, sighash) in sighashes.into_iter().enumerate() {
            let signature = self
                .signer
                .sign_message_tss_and_broadcast_to_other_guardians_and_wait_for_signature_sync(
                    sighash,
                )?;
            spend_transaction.inputs[i].script = signature.to_btc_script();
        }

        Ok(spend_transaction)
    }
}
