#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
extern crate sgx_tstd as std;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use serde_derive::Deserialize;
use std::collections::HashMap;
use teaclave_types;

pub fn verify_enclave_info<T, U>(enclave_info: &[u8], public_keys: &[T], signatures: &[U]) -> bool
where
    T: AsRef<[u8]>,
    U: AsRef<[u8]>,
{
    use ring::signature;

    for k in public_keys {
        let mut verified = false;
        for s in signatures {
            if signature::UnparsedPublicKey::new(&signature::RSA_PKCS1_2048_8192_SHA256, k)
                .verify(enclave_info, s.as_ref())
                .is_ok()
            {
                verified = true;
            }
        }
        if !verified {
            return false;
        }
    }

    true
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct EnclaveInfoToml(HashMap<String, teaclave_types::EnclaveMeasurement>);

pub fn load_enclave_info(
    content: &str,
) -> std::collections::HashMap<String, teaclave_types::EnclaveMeasurement> {
    let config: EnclaveInfoToml =
        toml::from_str(&content).expect("Content not correct, unable to load enclave info.");
    let mut info_map = std::collections::HashMap::new();
    for (k, v) in config.0 {
        info_map.insert(
            k,
            teaclave_types::EnclaveMeasurement::new(v.mr_enclave, v.mr_signer),
        );
    }

    info_map
}
