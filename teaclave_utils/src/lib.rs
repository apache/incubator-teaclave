#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
extern crate sgx_tstd as std;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use serde::Deserializer;
use serde_derive::Deserialize;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, UtilsError>;
use sgx_types::SGX_HASH_SIZE;

pub type SgxMeasurement = [u8; SGX_HASH_SIZE];

pub enum UtilsError {
    ParseError,
}

fn decode_hex_digit(digit: char) -> Result<u8> {
    match digit {
        '0'..='9' => Ok(digit as u8 - b'0'),
        'a'..='f' => Ok(digit as u8 - b'a' + 10),
        'A'..='F' => Ok(digit as u8 - b'A' + 10),
        _ => Err(UtilsError::ParseError),
    }
}

fn decode_hex(hex: &str) -> Result<Vec<u8>> {
    let mut r: Vec<u8> = Vec::new();
    let mut chars = hex.chars().enumerate();
    loop {
        let (_, first) = match chars.next() {
            None => break,
            Some(elt) => elt,
        };
        if first == ' ' {
            continue;
        }
        let (_, second) = chars.next().ok_or_else(|| UtilsError::ParseError)?;
        r.push((decode_hex_digit(first)? << 4) | decode_hex_digit(second)?);
    }
    Ok(r)
}

pub fn decode_spid(hex: &str) -> Result<sgx_types::sgx_spid_t> {
    let mut spid = sgx_types::sgx_spid_t::default();
    let hex = hex.trim();

    if hex.len() < 16 * 2 {
        return Err(UtilsError::ParseError);
    }

    let decoded_vec = decode_hex(hex)?;

    spid.id.copy_from_slice(&decoded_vec[..16]);

    Ok(spid)
}

pub fn percent_decode(orig: String) -> Result<String> {
    let v: Vec<&str> = orig.split('%').collect();
    let mut ret = String::new();
    ret.push_str(v[0]);
    if v.len() > 1 {
        for s in v[1..].iter() {
            let digit = u8::from_str_radix(&s[0..2], 16).map_err(|_| UtilsError::ParseError)?;
            ret.push(digit as char);
            ret.push_str(&s[2..]);
        }
    }
    Ok(ret)
}

/// Deserializes a hex string to a `SgxMeasurement` (i.e., [0; 32]).
pub fn from_hex<'de, D>(deserializer: D) -> std::result::Result<SgxMeasurement, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde::Deserialize;
    String::deserialize(deserializer).and_then(|string| {
        let v = decode_hex(&string).map_err(|_| Error::custom("ParseError"))?;
        let mut array = [0; 32];
        let bytes = &v[..array.len()]; // panics if not enough data
        array.copy_from_slice(bytes);
        Ok(array)
    })
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct EnclaveInfoToml(HashMap<String, EnclaveMeasurement>);

#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
pub struct EnclaveMeasurement {
    #[serde(deserialize_with = "from_hex")]
    pub mr_signer: SgxMeasurement,
    #[serde(deserialize_with = "from_hex")]
    pub mr_enclave: SgxMeasurement,
}

impl EnclaveMeasurement {
    pub fn new(mr_enclave: SgxMeasurement, mr_signer: SgxMeasurement) -> Self {
        Self {
            mr_enclave,
            mr_signer,
        }
    }
}

pub fn verify_enclave_info(enclave_info: &[u8], public_key: &[u8], signature: &[u8]) -> bool {
    use ring::signature;

    signature::UnparsedPublicKey::new(&signature::RSA_PKCS1_2048_8192_SHA256, public_key)
        .verify(enclave_info, signature)
        .is_ok()
}

pub fn load_enclave_info(content: &str) -> std::collections::HashMap<String, EnclaveMeasurement> {
    let config: EnclaveInfoToml =
        toml::from_str(&content).expect("Content not correct, unable to load enclave info.");
    let mut info_map = std::collections::HashMap::new();
    for (k, v) in config.0 {
        info_map.insert(k, EnclaveMeasurement::new(v.mr_enclave, v.mr_signer));
    }

    info_map
}
