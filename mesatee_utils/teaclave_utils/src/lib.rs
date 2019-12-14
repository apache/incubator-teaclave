#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
extern crate sgx_tstd as std;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

#[macro_use]
extern crate log;
use serde::Deserializer;
use serde_derive::Deserialize;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, UtilsError>;
type SgxMeasure = [u8; 32];

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

pub fn decode_hex(hex: &str) -> Result<Vec<u8>> {
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
        debug!("Input spid file len ({}) is incorrect!", hex.len());
        return Ok(spid);
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

/// Deserializes a hex string to a `SgxMeasure` (i.e., [0; 32]).
pub fn from_hex<'de, D>(deserializer: D) -> std::result::Result<SgxMeasure, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde::Deserialize;
    String::deserialize(deserializer).and_then(|string| {
        let v = decode_hex(&string).map_err(|_| Error::custom("ParseError"))?;
        let mut array: SgxMeasure = [0; 32];
        let bytes = &v[..array.len()]; // panics if not enough data
        array.copy_from_slice(bytes);
        Ok(array)
    })
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct EnclaveInfoToml(HashMap<String, EnclaveInfo>);

#[derive(Debug, Deserialize)]
struct EnclaveInfo {
    #[serde(deserialize_with = "from_hex")]
    mrsigner: SgxMeasure,
    #[serde(deserialize_with = "from_hex")]
    enclave_hash: SgxMeasure,
}

// This function fails when enclave info signatures mismatch hard-coded
// auditor keys. We expect the program to crash in those cases
pub fn load_and_verify_enclave_info(
    enclave_info_file_path: &std::path::Path,
    // A vector of signer meta info, each tuple is
    // (harded-coded public key, file path to signature of enclave_info.toml)
    enclave_signers: &[(&[u8], &std::path::Path)],
) -> std::collections::HashMap<String, (SgxMeasure, SgxMeasure)> {
    #[cfg(not(feature = "mesalock_sgx"))]
    use std::fs::{self, File};
    #[cfg(feature = "mesalock_sgx")]
    use std::untrusted::fs::{self, File};

    use std::io::Read;

    let content = fs::read_to_string(enclave_info_file_path)
        .unwrap_or_else(|_| panic!("cannot find enclave info at {:?}", enclave_info_file_path));

    // verify autenticity of enclave identity info
    for signer in enclave_signers {
        let mut sig_file = File::open(signer.1)
            .unwrap_or_else(|_| panic!("cannot find signature file at {:?}", signer.1));
        let mut sig = Vec::<u8>::new();
        sig_file
            .read_to_end(&mut sig)
            .unwrap_or_else(|_| panic!("cannot read signature from {:?}", signer.1));

        use ring::signature;
        let public_key =
            signature::UnparsedPublicKey::new(&signature::RSA_PKCS1_2048_8192_SHA256, signer.0);
        public_key
            .verify(content.as_bytes(), sig.as_slice())
            .expect("invalid signature for enclave info file");
    }
    let config: EnclaveInfoToml =
        toml::from_str(&content).expect("Content not correct, unable to load enclave info.");
    let mut info_map = std::collections::HashMap::new();
    for (k, v) in config.0 {
        info_map.insert(k, (v.mrsigner, v.enclave_hash));
    }

    info_map
}
