// DATA INSTRUCTIONS
const OP_PUSHBYTES_32: u8 = 0x20;
const OP_PUSHBYTES_20: u8 = 0x14;
const OP_PUSH1: u8 = 0x01;
const OP_PUSHDATA1: u8 = 0x4c;

// OP Codes
const OP_0: u8 = 0x00;
const OP_1: u8 = 0x51;

const OP_DUP: u8 = 0x76;
const OP_EQUALVERIFY: u8 = 0x88;
const OP_DROP: u8 = 0x75;
const OP_2DROP: u8 = 0x6d;

const OP_HASH160: u8 = 0xa9;
const OP_CHECKSIG: u8 = 0xac;

/*
const PLACEHOLDER_EMITTER_ADDRESS: [u8; 32] = [0x69u8; 32];
const PLACEHOLDER_SUB_ADDRESS: [u8; 32] = [0x22u8; 32];
const PLACEHOLDER_WORMHOLE_PUBLIC_KEY_HASH: [u8; 20] = [0x33u8; 20];


const PLACEHOLDER_EMITTER_ADDRESS_INDEX: usize = 1;
const PLACEHOLDER_SUB_ADDRESS_INDEX: usize = 35;
const PLACEHOLDER_WORMHOLE_PUBLIC_KEY_HASH_INDEX: usize = 91;
const WORMHOLE_SPENDSCRIPT_TPL: [u8; SIZE_OF_WORMHOLE_SPENDSCRIPT_TPL] = crate::const_concat_arrays!(
    [OP_PUSHBYTES_32],// 1
    PLACEHOLDER_EMITTER_ADDRESS, //  33
    [OP_2DROP], // 34
    [OP_PUSHBYTES_32],// 35
    PLACEHOLDER_SUB_ADDRESS,  // 67
    [OP_DROP], // 68
    [OP_DUP], // 69
    [OP_HASH160], // 70
    [OP_PUSHBYTES_20], // 71
    PLACEHOLDER_WORMHOLE_PUBLIC_KEY_HASH, // 91
    [OP_EQUALVERIFY], // 92
    [OP_CHECKSIG] // 93
);

*/


const SIZE_OF_WORMHOLE_SPENDSCRIPT_TPL: usize = 93;
fn btc_script_push_number(x: u16) -> Vec<u8> {
    if x == 0 {
        return vec![OP_0];
    }else if x <= 16 {
        // OP_1 to OP_16
        vec![OP_1 - 1 + (x as u8)]
    } else if x <= 0xff {
        // OP_PUSH1 + 1 byte
        vec![OP_PUSH1, x as u8] 
    } else {
        // OP_PUSHDATA1 + 2 bytes
        vec![OP_PUSHDATA1, 0x02, (x & 0xff) as u8, (x >> 8) as u8]
    }
}

const fn btc_script_size_of_push_number(x: u16) -> usize {
    if x <= 16 {
        1 // OP_0 to OP_16
    } else if x <= 0xff {
        2 // OP_PUSH1 + 1 byte
    } else {
        3 // OP_PUSHDATA1 + 2 bytes
    }
}


pub fn construct_wormhole_vaa_script(
    emitter_chain: u16,
    emitter_contract_address: &[u8; 32],
    sub_address_seed: &[u8; 32],
    guardian_public_key_hash: &[u8; 20]
) -> Vec<u8> {
    let script_size = SIZE_OF_WORMHOLE_SPENDSCRIPT_TPL+ btc_script_size_of_push_number(emitter_chain);
    let mut data = Vec::with_capacity(script_size);

    data.extend_from_slice(&btc_script_push_number(emitter_chain));
    data.extend_from_slice(&[OP_PUSHBYTES_32]);
    data.extend_from_slice(emitter_contract_address);
    data.extend_from_slice(&[OP_2DROP, OP_PUSHBYTES_32]);
    data.extend_from_slice(sub_address_seed);
    data.extend_from_slice(&[OP_DROP,OP_DUP,OP_HASH160,OP_PUSHBYTES_20]);
    data.extend_from_slice(guardian_public_key_hash);
    data.extend_from_slice(&[OP_EQUALVERIFY, OP_CHECKSIG]);

    data

}

