# Integrating a Threshold Signature Scheme (TSS)

This document provides a guide for integrating a multi-party Threshold Signature Scheme (TSS) into the VAA-P2SH Guardian processor. The library is designed with a trait-based approach, making it straightforward to replace the example in-memory signer with a production-grade, distributed TSS implementation.

## 1. Core Integration Points

The primary abstraction for signing is the `WormholeTSSSignerAsync` trait located in `src/wormhole/tss_signer.rs`.

```rust
#[async_trait::async_trait]
pub trait WormholeTSSSignerAsync: WithTSSPublicKey {
    // This will perform the signing contribution for the guardian, broadcast the 
    // contribution to other guardians, wait until the threshold of guardian 
    // signatures has been collected, and return the final signature.
    async fn sign_message_tss_and_broadcast_to_other_guardians_and_wait_for_signature(
        &self, 
        message: QHash256
    ) -> anyhow::Result<PsyCompressedSecp256K1Signature>;
}

pub trait WithTSSPublicKey {
    fn get_tss_public_key(&self) -> CompressedPublicKey;
}
```

Your task is to create a struct that implements this trait and contains the logic for your specific TSS protocol (e.g., GG20, FROST).

## 2. Step-by-Step Integration Guide

### Step 1: Define Your TSS Signer Struct

First, create a struct that will manage the state of your TSS node. This includes peer-to-peer networking, cryptographic material, and the state machine for the TSS signing ceremony.

```rust
use psy_doge_bridge_wormhole::wormhole::tss_signer::{WormholeTSSSignerAsync, WithTSSPublicKey};
use psy_doge_bridge_wormhole::secp256k1::signer::CompressedPublicKey;
// ... other necessary imports for your TSS library and networking stack

// Example struct for your TSS implementation
pub struct MyTssGuardianNode {
    // The shared public key for the entire guardian set
    shared_public_key: CompressedPublicKey,
    // Your TSS party ID
    party_id: u16,
    // Threshold required for signing
    threshold: u16,
    // P2P networking client to communicate with other guardians
    p2p_client: MyP2pClient,
    // Key share material for this guardian
    key_share: MyTssKeyShare,
    // ... any other state needed
}

impl MyTssGuardianNode {
    pub fn new(...) -> Self {
        // Constructor logic
    }
}
```

### Step 2: Implement the `WithTSSPublicKey` Trait

This trait has one method, `get_tss_public_key`, which must return the single, shared compressed public key (`[u8; 33]`) of the entire Guardian TSS group. This public key's hash is what is embedded in the VAA-P2SH script.

```rust
impl WithTSSPublicKey for MyTssGuardianNode {
    fn get_tss_public_key(&self) -> CompressedPublicKey {
        self.shared_public_key
    }
}
```

### Step 3: Implement the `WormholeTSSSignerAsync` Trait

This is the core of the integration. You need to implement the `sign_message_tss_and_broadcast_to_other_guardians_and_wait_for_signature` method.

This method receives the `message`, which is the 32-byte **sighash** of the Dogecoin transaction that needs to be signed.

```rust
use async_trait::async_trait;
use psy_doge_bridge_wormhole::dogecoin::hash::QHash256;
use psy_doge_bridge_wormhole::secp256k1::signature::PsyCompressedSecp256K1Signature;

#[async_trait]
impl WormholeTSSSignerAsync for MyTssGuardianNode {
    async fn sign_message_tss_and_broadcast_to_other_guardians_and_wait_for_signature(
        &self,
        message: QHash256,
    ) -> anyhow::Result<PsyCompressedSecp256K1Signature> {
        
        // 1. Initiate the TSS signing ceremony with the message (sighash).
        // This will likely involve creating a state machine for the signing rounds.
        let signing_ceremony = self.p2p_client.initiate_signing(
            &self.key_share,
            &message,
            self.party_id,
            self.threshold,
        );

        // 2. Participate in the distributed signing rounds.
        // This is highly specific to your TSS library. It will involve sending
        // and receiving messages with other guardians.
        let raw_signature_result = signing_ceremony.execute().await;

        // 3. Reconstruct the final signature.
        // The TSS protocol will output the r and s values of the ECDSA signature.
        let (r_bytes, s_bytes) = match raw_signature_result {
            Ok(sig) => (sig.r, sig.s),
            Err(e) => anyhow::bail!("TSS signing ceremony failed: {:?}", e),
        };

        // 4. Format the signature into the required struct.
        // The signature must be in [r, s] format (64 bytes total).
        let mut final_signature = [0u8; 64];
        final_signature[0..32].copy_from_slice(&r_bytes);
        final_signature[32..64].copy_from_slice(&s_bytes);

        Ok(PsyCompressedSecp256K1Signature {
            public_key: self.shared_public_key.0,
            signature: final_signature,
            message,
        })
    }
}
```

### Step 4: Instantiate the Guardian Processor

Once your `MyTssGuardianNode` is implemented, you can instantiate the `WormholeGuardianProcessorAsync` with it. The processor will handle all the VAA validation, UTXO fetching, and sighash generation, and will call your TSS signer when it's time to sign.

```rust
// In your guardian service's main logic:

// 1. Initialize your Dogecoin RPC provider
let rpc_provider = DogeLinkElectrsRPCAsync::<DogeTestNetConfig>::new("...");
let wh_rpc_provider = DogecoinTransactionProviderWithCache::new(rpc_provider);

// 2. Initialize your custom TSS signer
let tss_signer = MyTssGuardianNode::new(/* ... configuration ... */);

// 3. The library can calculate the public key hash for you...
let guardian_processor = WormholeGuardianProcessorAsync::new::<CommonDogeHashProvider>(
    wh_rpc_provider,
    tss_signer,
);

// ...or you can provide it directly if it's a known constant.
// let tss_pubkey_hash = Hasher::bitcoin_hash160(&tss_signer.get_tss_public_key().0);
// let guardian_processor = WormholeGuardianProcessorAsync::new_with_tss_public_key_hash(
//     wh_rpc_provider,
//     tss_signer,
//     tss_pubkey_hash
// );

// 4. Process a VAA message (this would be received from the Wormhole network)
let vaa_message = ...; // Your logic to deserialize a VAA
let signed_tx = guardian_processor
    .validate_p2sh_vaa_message_and_sign_async::<CommonDogeHashProvider, DogeTestNetConfig>(vaa_message)
    .await?;

// 5. Broadcast the transaction
// ...
```

By following these steps, you can plug any compliant TSS signing logic directly into the Psy-Wormhole bridge framework, enabling fully distributed, secure control over Dogecoin assets.