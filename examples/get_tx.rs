use psy_doge_bridge_wormhole::{dogecoin::{constants::DogeTestNetConfig, hash::CommonDogeHashProvider}, psy_doge_link::link_async::DogeLinkElectrsRPCAsync, tx_store::traits::DogecoinRPCProviderAsync};


type Hasher = CommonDogeHashProvider;
type Network = DogeTestNetConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let txid = hex_literal::hex!(
        "0fbe23bb45d62080672425f8bdce8fa2cd439e7c7cae5d3e1d35503e76fef8d9"
    );

    let rpc_provider = DogeLinkElectrsRPCAsync::<Network>::new("https://doge-electrs-testnet-demo.qed.me");
    let tx = rpc_provider.get_transaction_by_txid(&txid).await?;

    let got_txid = tx.get_txid::<Hasher>();
    println!("expected txid: {}", hex::encode(txid));
    println!("got txid:      {}", hex::encode(got_txid));


    Ok(())
}