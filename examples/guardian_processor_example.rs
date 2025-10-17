use psy_doge_bridge_wormhole::{
    dogecoin::{
        address::{AddressToBTCScript, BTCAddress160}, constants::{DogeNetworkConfig, DogeTestNetConfig}, hash::{CommonDogeHashProvider, QHash160, QHash256}, sighash::{SigHashPreimage, SIGHASH_ALL}, transaction::{
            BTCTransaction, BTCTransactionInputWithoutScript,
            BTCTransactionOutput,
        }
    }, psy_doge_link::{link_async::DogeLinkElectrsRPCAsync, traits::PsyBitcoinAPIAsync}, secp256k1::signer::{
        CompressedPublicKeyToP2PKH, MemorySecp256K1Wallet, Secp256K1WalletProvider, SimpleSinglePublicKeySigner, SimpleSingleSigner
    }, tx_store::{memory_cache_transaction_store::DogecoinTransactionProviderWithCache, traits::DogecoinRPCProviderAsync}, wormhole::{
        guardian_processor::WormholeGuardianProcessorAsync,
        p2sh_vaa::{WormholeBitcoinLikeVAAMessage, WormholeBitcoinLikeVAAMetadata},
    }
};
use tokio::time::sleep;
use std::time::Duration;

type Hasher = CommonDogeHashProvider;
type Network = DogeTestNetConfig;

pub fn create_p2pkh_tx<W: Secp256K1WalletProvider>(
    wallet: &W,
    address: QHash160,
    inputs: &[BTCTransactionInputWithoutScript],
    outputs: Vec<BTCTransactionOutput>,
) -> anyhow::Result<BTCTransaction> {
    let inputs_len = inputs.len();
    let script = BTCAddress160::new_p2pkh(address).to_btc_script();

    let mut base_tx = BTCTransaction::from_partial(inputs, outputs);
    let sighashes: Vec<QHash256> = (0..inputs_len)
        .map(|i| {
            SigHashPreimage::for_transaction_pre_segwit(&base_tx, i, &script, SIGHASH_ALL)
                .get_hash::<Hasher>()
        })
        .collect();

    let public_key = wallet
        .get_public_key_for_p2pkh(&address)
        .ok_or_else(|| anyhow::anyhow!("public key not found"))?;

    for i in 0..inputs_len {
        base_tx.inputs[i].script = wallet.sign(&public_key, sighashes[i])?.to_btc_script();
    }

    Ok(base_tx)
}

async fn fund_script<N: DogeNetworkConfig, RPC: PsyBitcoinAPIAsync, W: Secp256K1WalletProvider>(wallet: &W, rpc: &RPC, from_private_key: [u8; 32], to_address: BTCAddress160, amount: u64, fee: u64) -> anyhow::Result<QHash256> {
    let signer = SimpleSinglePublicKeySigner::new_insecure_memory_signer_with_private_key::<Hasher>(from_private_key)?;

    let from_address = signer.get_compressed_public_key().to_p2pkh_address::<Hasher>();
    let from_address = BTCAddress160::new_p2pkh(from_address);

    let utxos = rpc.get_utxos(from_address).await?;
    let mut inputs = Vec::new();
    let mut input_value = 0u64;
    for utxo in utxos.iter() {
        let mut hash = utxo.txid.clone();
        hash.reverse();
        inputs.push(BTCTransactionInputWithoutScript {
            hash,
            index: utxo.vout,
            sequence: 0xffffffff,
        });
        input_value += utxo.value;
        if input_value >= amount + fee {
            break;
        }
    }

    if input_value < amount + fee {
        anyhow::bail!("Insufficient funds to fund script");
    }

    let outputs = vec![
        BTCTransactionOutput {
            value: amount,
            script: to_address.to_btc_script(),
        },
        BTCTransactionOutput {
            value: input_value - amount - fee,
            script: from_address.to_btc_script(),
        },
    ];

    let tx = create_p2pkh_tx(wallet, from_address.address, &inputs, outputs)?;
    let result_txid = rpc.send_transaction(&tx).await?;
    Ok(result_txid)
}
async fn setup_scenario(fund_to: BTCAddress160, amount: u64) -> anyhow::Result<QHash256> {
    // if this isn't working, get some dogecoin testnet coins for np26V1nCcAjmDhzsE3jYgmHrn511eunx5f
    // You can do this at our testnet faucet: https://faucet.doge.toys
    let funder_private_key = hex_literal::hex!(
        "2220d9bd068247243fbd53c04824ae60e7e162ee6823a4cb809b8b725c32c6d6"
    );

    let mut wallet = MemorySecp256K1Wallet::new();
    let public_key = wallet.add_private_key::<Hasher>(funder_private_key)?;
    let funder_address = public_key.to_p2pkh_address::<Hasher>();
    let funder_address = BTCAddress160::new_p2pkh(funder_address);
    println!(
        "Funder address: {}",
        funder_address.to_address_string::<Network>()
    );

    let rpc_provider = DogeLinkElectrsRPCAsync::<Network>::new("https://doge-electrs-testnet-demo.qed.me");
    let txid = fund_script::<Network, _, _>(&wallet, &rpc_provider, funder_private_key, fund_to, amount, 100_000).await?;
    println!("Funded script with TXID: {}", hex::encode(txid));





    Ok(txid)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Setup a signer (in a real scenario, this would be a TSS signer)
    // For this example, we use a simple in-memory signer with a fixed private key.


    

    let tss_private_key = hex_literal::hex!(
        "6cbcc00ade99f6b69b62aa222b5ea5e993faf849bdccbedbef8d40d9d64e6491"
    );

    let signer =
        SimpleSinglePublicKeySigner::new_insecure_memory_signer_with_private_key::<Hasher>(
            tss_private_key,
        )?;

    // The guardian set's public key, derived from the signer.
    let guardian_pubkey = signer.get_compressed_public_key();
    let guardian_pubkey_hash: QHash160 = guardian_pubkey.to_p2pkh_address::<Hasher>();

    // 2. Define the VAA message metadata.
    // This data would typically come from a Wormhole VAA.
    let vaa_metadata = WormholeBitcoinLikeVAAMetadata {
        emitter_chain: 1, // Example: Solana
        emitter_contract_address: [1u8; 32],
        sub_address_seed: [2u8; 32],
        total_output_amount: 900_000,      // 0.009 DOGE
        max_doge_transaction_fee: 1000_000, // 0.001 DOGE
        min_doge_transaction_fee: 100_000,  // 0.0001 DOGE
    };

    // 3. Determine the P2SH address where funds are locked.
    let vaa_p2sh_address = vaa_metadata.get_p2sh_address::<Network, Hasher>(&guardian_pubkey_hash);
    println!(
        "Funds should be locked in P2SH address: {}",
        vaa_p2sh_address.to_address_string::<Network>()
    );

    let funding_txid = setup_scenario(vaa_p2sh_address, vaa_metadata.total_output_amount + vaa_metadata.max_doge_transaction_fee).await?;

    let rpc_provider = DogeLinkElectrsRPCAsync::<Network>::new("https://doge-electrs-testnet-demo.qed.me");
    let wh_rpc_provider = DogecoinTransactionProviderWithCache::new(rpc_provider.clone());

    loop {
        match rpc_provider.get_transaction_with_status(&funding_txid).await {
            Ok(tx) => {
                if !tx.status.confirmed {
                    println!("Funding transaction found, but not yet confirmed, waiting...");
                }else{
                    println!("Funding transaction found: {}", hex::encode(funding_txid));
                    println!("Funding transaction details: {:#?}", tx);
                }
                break;
            }
            Err(_) => {
                println!("Funding transaction not yet mined, waiting...");
                sleep(Duration::from_millis(5_000)).await;
            }
        }
    };

    let input_hash = {
        let mut h = funding_txid;
        h.reverse();
        h
    };

    // 5. Define the VAA message that will spend the locked funds.
    let vaa_message = WormholeBitcoinLikeVAAMessage {
        metadata: vaa_metadata,
        inputs: vec![BTCTransactionInputWithoutScript::new_simple(
            input_hash,
            0,
        )],
        outputs: vec![BTCTransactionOutput {
            value: 900_000, // 0.009 DOGE to recipient
            script: BTCAddress160::new_p2pkh([0xbb; 20]) // Dummy recipient address
                .to_btc_script(),
        }],
    };

    // 6. Initialize the Wormhole Guardian Processor.
    let guardian_processor =
        WormholeGuardianProcessorAsync::new_with_tss_public_key_hash(wh_rpc_provider.clone(), signer, guardian_pubkey_hash);

    // 7. Validate the message and generate the signed transaction.
    println!("Validating VAA message and signing...");
    let signed_tx = guardian_processor
        .validate_p2sh_vaa_message_and_sign_async::<Hasher, Network>(vaa_message)
        .await?;

    println!("\nSuccessfully signed transaction!");
    println!(
        "Transaction ID: {}",
        hex::encode(signed_tx.get_txid::<Hasher>())
    );
    println!("Raw Transaction Hex: \n{}", hex::encode(signed_tx.to_bytes()));

    wh_rpc_provider.submit_raw_transaction(&signed_tx.to_bytes()).await?;
    println!("Transaction broadcasted successfully.");


    Ok(())
}