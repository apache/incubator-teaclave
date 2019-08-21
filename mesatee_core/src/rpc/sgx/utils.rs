// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::Error;
use crate::ErrorKind;
use crate::Result;

fn decode_hex_digit(digit: char) -> Result<u8> {
    match digit {
        '0'..='9' => Ok(digit as u8 - b'0'),
        'a'..='f' => Ok(digit as u8 - b'a' + 10),
        'A'..='F' => Ok(digit as u8 - b'A' + 10),
        _ => Err(Error::from(ErrorKind::ParseError)),
    }
}

#[cfg(feature = "mesalock_sgx")]
pub(crate) fn decode_hex(hex: &str) -> Result<Vec<u8>> {
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
        let (_, second) = chars
            .next()
            .ok_or_else(|| Error::from(ErrorKind::ParseError))?;
        r.push((decode_hex_digit(first)? << 4) | decode_hex_digit(second)?);
    }
    Ok(r)
}

#[cfg(feature = "mesalock_sgx")]
pub(crate) fn decode_spid(hex: &str) -> Result<sgx_types::sgx_spid_t> {
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

#[cfg(feature = "mesalock_sgx")]
pub(crate) fn percent_decode(orig: String) -> Result<String> {
    let v: Vec<&str> = orig.split('%').collect();
    let mut ret = String::new();
    ret.push_str(v[0]);
    if v.len() > 1 {
        for s in v[1..].iter() {
            let digit =
                u8::from_str_radix(&s[0..2], 16).map_err(|_| Error::from(ErrorKind::ParseError))?;
            ret.push(digit as char);
            ret.push_str(&s[2..]);
        }
    }
    Ok(ret)
}

use super::SgxMeasure;

fn decode_sgx_measure(lines: [&str; 2]) -> Result<SgxMeasure> {
    let decoded_hash: Vec<u8> = lines
        .iter()
        .map(|line| line.trim().split(' '))
        .flatten()
        .try_fold(Vec::<u8>::new(), |mut r, hex| -> Result<Vec<u8>> {
            // skip "0x"
            let mut hex = hex.chars().skip(2);

            use super::fail::MayfailNop;
            let byte = mayfail! {
                first =<< hex.next();
                second =<< hex.next();
                first_digit =<< decode_hex_digit(first);
                second_digit =<< decode_hex_digit(second);
                ret first_digit * 16 + second_digit
            }?;
            r.push(byte);
            Ok(r)
        })?;
    let mut sgx_measure = [0u8; 32];
    sgx_measure[..].copy_from_slice(decoded_hash.as_slice());

    Ok(sgx_measure)
}

// This function fails when enclave info signatures mismatch hard-coded
// auditor keys. We expect the program to crash in those cases
pub fn load_and_verify_enclave_info(
    enclave_info_file_path: &std::path::Path,
    // A vector of signer meta info, each tuple is
    // (harded-coded public key, file path to signature of enclave_info.txt)
    enclave_signers: &[(&[u8], &std::path::Path)],
) -> std::collections::HashMap<String, (SgxMeasure, SgxMeasure)> {
    #[cfg(not(feature = "mesalock_sgx"))]
    use std::fs::File;
    #[cfg(feature = "mesalock_sgx")]
    use std::untrusted::fs::File;

    use std::io::Read;

    let mut content = String::new();
    let mut f_enclave_info = File::open(enclave_info_file_path)
        .unwrap_or_else(|_| panic!("cannot find enclave info at {:?}", enclave_info_file_path));
    f_enclave_info
        .read_to_string(&mut content)
        .expect("cannot read from enclave info file");

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
            .expect("aa");
    }

    let lines: Vec<&str> = content.split('\n').collect();
    let mut info_map = std::collections::HashMap::new();
    lines
        .as_slice()
        .chunks_exact(7)
        .try_for_each(|group| -> Result<()> {
            let name = String::from(group[0]);

            let mr_signer = decode_sgx_measure([group[2], group[3]])?;
            let mr_enclave = decode_sgx_measure([group[5], group[6]])?;

            info_map.insert(name, (mr_signer, mr_enclave));

            Ok(())
        })
        .expect("malformed enclave info file");

    info_map
}
