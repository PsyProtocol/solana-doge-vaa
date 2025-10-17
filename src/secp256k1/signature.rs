use crate::dogecoin::hash::QHash256;


#[derive(PartialEq, Clone, Copy, Debug)]
pub struct PsyCompressedSecp256K1Signature {
    pub public_key: [u8; 33],

    pub signature: [u8; 64],

    pub message: QHash256,
}

pub fn u256_to_der(u256: &[u8]) -> Vec<u8> {
    assert_eq!(u256.len(), 32);
    let mut result = vec![];
    result.push(0x02u8);
    if (u256[0] & 0x80) != 0 {
        result.push((u256.len() + 1) as u8);
        result.push(0);
        result.extend_from_slice(u256);
    } else {
        result.push(u256.len() as u8);
        result.extend_from_slice(u256);
    }
    result
}

impl PsyCompressedSecp256K1Signature {
    pub fn to_btc_script(&self) -> Vec<u8> {
        let r = u256_to_der(&self.signature[0..32]);
        let s = u256_to_der(&self.signature[32..64]);
        let combined_rs_length = (r.len() + s.len()) as u8;
        let sig_stack_raw = [
            vec![combined_rs_length + 3, 0x30u8, combined_rs_length],
            r,
            s,
            vec![0x01],
        ]
        .concat();
        [sig_stack_raw, vec![0x21], self.public_key.to_vec()].concat()
    }
}