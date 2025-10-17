
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error as SerdeError;
use std::collections::HashMap;
use crate::dogecoin::{hash::QHash256, transaction::BTCTransaction};
use crate::psy_doge_link::bytes::U8BytesFixed;

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct BTCFeeRateEstimate {
    pub feerate: f64,
    pub blocks: i32,
}
impl BTCFeeRateEstimate {
    pub fn to_feerate_u64(&self) -> u64 {
        if self.feerate <= 0.0f64 {
            1
        } else {
            (self.feerate * 100_000_000.0) as u64
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct BTCUTXOStatus {
    #[serde(default)]
    pub block_hash: QHash256,
    #[serde(default)]
    pub block_height: u64,
    #[serde(default)]
    pub block_time: u64,
    pub confirmed: bool,
}
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct BTCUTXO {
    pub status: BTCUTXOStatus,
    pub txid: QHash256,
    pub value: u64,
    pub vout: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct PartialBTCUTXO {
    pub txid: QHash256,
    pub value: u64,
    pub vout: u32,
}

#[derive(PartialEq, Clone, Debug)]
pub struct BTCTransactionWithVout {
    pub transaction: BTCTransaction,
    pub vout: u32,
}

impl From<BTCUTXO> for PartialBTCUTXO {
    fn from(utxo: BTCUTXO) -> Self {
        Self {
            txid: utxo.txid,
            value: utxo.value,
            vout: utxo.vout,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct BTCOutpoint {
    pub txid: QHash256,
    pub vout: u32,
}


// New structs for Electrs API responses
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeeEstimateMap(pub HashMap<String, f64>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ElectrsUTXOStatus {
    pub confirmed: bool,
    #[serde(default)]
    pub block_height: u64,
    #[serde(default)]
    pub block_hash: Option<U8BytesFixed<32>>,
    #[serde(default)]
    pub block_time: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ElectrsUTXO {
    #[serde(deserialize_with = "hex_to_qhash256")]
    pub txid: QHash256,
    pub vout: u32,
    pub status: ElectrsTxStatus,
    pub value: u64,
}


impl From<ElectrsUTXO> for BTCUTXO {
    fn from(utxo: ElectrsUTXO) -> Self {
        let mut block_hash = [0u8; 32];
        if let Some(hash) = utxo.status.block_hash {
            block_hash.copy_from_slice(&hash);
        }

        Self {
            txid: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
            status: BTCUTXOStatus {
                confirmed: utxo.status.confirmed,
                block_height: utxo.status.block_height.unwrap_or(0),
                block_hash,
                block_time: utxo.status.block_time.unwrap_or(0),
            },
        }
    }
}

/* 
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::dogecoin::{hash::QHash256, transaction::BTCTransaction};

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct BTCFeeRateEstimate {
    pub feerate: f64,
    pub blocks: i32,
}
impl BTCFeeRateEstimate {
    pub fn to_feerate_u64(&self) -> u64 {
        if self.feerate <= 0.0f64 {
            1
        } else {
            (self.feerate * 100_000_000.0) as u64
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct BTCUTXOStatus {
    #[serde(default)]
    #[serde_as(as = "serde_with::hex::Hex")]
    pub block_hash: QHash256,
    #[serde(default)]
    pub block_height: u64,
    #[serde(default)]
    pub block_time: u64,
    pub confirmed: bool,
}
#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct BTCUTXO {
    pub status: BTCUTXOStatus,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub txid: QHash256,
    pub value: u64,
    pub vout: u32,
}

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct PartialBTCUTXO {
    #[serde_as(as = "serde_with::hex::Hex")]
    pub txid: QHash256,
    pub value: u64,
    pub vout: u32,
}

#[derive(PartialEq, Clone, Debug)]
pub struct BTCTransactionWithVout {
    pub transaction: BTCTransaction,
    pub vout: u32,
}

impl From<BTCUTXO> for PartialBTCUTXO {
    fn from(utxo: BTCUTXO) -> Self {
        Self {
            txid: utxo.txid,
            value: utxo.value,
            vout: utxo.vout,
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct BTCOutpoint {
    #[serde_as(as = "serde_with::hex::Hex")]
    pub txid: QHash256,
    pub vout: u32,
}*/



fn hex_to_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    hex::decode(s).map_err(SerdeError::custom)
}

fn hex_to_qhash256<'de, D>(deserializer: D) -> Result<QHash256, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let mut bytes = [0u8; 32];
    hex::decode_to_slice(s, &mut bytes).map_err(SerdeError::custom)?;
    Ok(bytes)
}

fn deserialize_optional_qhash256<'de, D>(deserializer: D) -> Result<Option<QHash256>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Deserialize::deserialize(deserializer)?;
    match s {
        Some(s) => {
            let mut bytes = [0u8; 32];
            hex::decode_to_slice(&s, &mut bytes)
                .map_err(SerdeError::custom)?;
            Ok(Some(bytes))
        }
        None => Ok(None),
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ElectrsTxVout {
    #[serde(deserialize_with = "hex_to_bytes")]
    pub scriptpubkey: Vec<u8>,
    pub value: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ElectrsTxVin {
    #[serde(deserialize_with = "hex_to_qhash256")]
    pub txid: QHash256,
    pub vout: u32,
    #[serde(deserialize_with = "hex_to_bytes")]
    pub scriptsig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ElectrsTxStatus {
    pub confirmed: bool,
    pub block_height: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_optional_qhash256")]
    pub block_hash: Option<QHash256>,
    pub block_time: Option<u64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ElectrsGetTxResponse {
    pub version: u32,
    pub locktime: u32,
    pub vin: Vec<ElectrsTxVin>,
    pub vout: Vec<ElectrsTxVout>,
}
