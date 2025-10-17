use crate::{dogecoin::hash::QHash256, secp256k1::{signature::PsyCompressedSecp256K1Signature, signer::{CompressedPublicKey, SimpleSingleSigner}}};


pub trait WithTSSPublicKey {
    fn get_tss_public_key(&self) -> CompressedPublicKey;
}
#[async_trait::async_trait]
pub trait WormholeTSSSignerAsync: WithTSSPublicKey {
    // this will perform the signing contribution for the guardian and broadcast the contribution to other guardians
    // this waits until the threshold of guardian signatures have been collected and returns the final signature
    async fn sign_message_tss_and_broadcast_to_other_guardians_and_wait_for_signature(&self, message: QHash256) -> anyhow::Result<PsyCompressedSecp256K1Signature>;
}


pub trait WormholeTSSSignerSync: WithTSSPublicKey {
    // this will perform the signing contribution for the guardian and broadcast the contribution to other guardians
    // this waits until the threshold of guardian signatures have been collected and returns the final signature
    fn sign_message_tss_and_broadcast_to_other_guardians_and_wait_for_signature_sync(&self, message: QHash256) -> anyhow::Result<PsyCompressedSecp256K1Signature>;
}
impl<T: SimpleSingleSigner> WithTSSPublicKey for T {
    fn get_tss_public_key(&self) -> CompressedPublicKey {
        self.get_compressed_public_key()
    }
}


// for local development/debugging, we can use a single signer implementation, like an in memory secp256k1 provider
#[async_trait::async_trait]
impl<T: SimpleSingleSigner + Sync> WormholeTSSSignerAsync for T {
    async fn sign_message_tss_and_broadcast_to_other_guardians_and_wait_for_signature(&self, message: QHash256) -> anyhow::Result<PsyCompressedSecp256K1Signature> {
        self.sign_message(message)
    }
}

// for local development/debugging, we can use a single signer implementation, like an in memory secp256k1 provider
impl<T: SimpleSingleSigner> WormholeTSSSignerSync for T {
    fn sign_message_tss_and_broadcast_to_other_guardians_and_wait_for_signature_sync(&self, message: QHash256) -> anyhow::Result<PsyCompressedSecp256K1Signature> {
        self.sign_message(message)
    }
}