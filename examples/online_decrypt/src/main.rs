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

use lazy_static::lazy_static;
use mesatee_sdk::{Mesatee, MesateeEnclaveInfo};
use ring::aead::{self, Aad, BoundKey, Nonce, UnboundKey};
use ring::rand;
use ring::rand::SecureRandom;
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{env, fs};

static FUNCTION_NAME: &str = "decrypt";
static USER_ID: &str = "uid";
static USER_TOKEN: &str = "token";

lazy_static! {
    static ref TMS_ADDR: SocketAddr = "127.0.0.1:5554".parse().unwrap();
    static ref TDFS_ADDR: SocketAddr = "127.0.0.1:5065".parse().unwrap();
}

fn print_usage() {
    let msg = "
    ./online_decrypt gen_and_upload_key key_saving_path key_file_id_saving_path
    ./online_decrypt local_encrypt plaintxt_path encrypted_data_saving_path key_path
    ./online_decrypt online_decrypt encrypted_data_path key_file_id decrypted_data_saving_path

    gen_and_upload_key: Generate key; Upload it to TDFS
    local_encrypt: Encrypt data and save it;
    remote_decrypt: Invoke MesaTEE to decrypt file
    ";
    println!("usage: \n{}", msg);
}

#[derive(Serialize, Deserialize)]
struct AEADKeyConfig {
    pub key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ad: Vec<u8>,
}

impl AEADKeyConfig {
    pub fn new() -> Self {
        let mut key_config = AEADKeyConfig {
            key: vec![0; 32],
            nonce: vec![0; 12],
            ad: vec![0; 5],
        };

        let rng = rand::SystemRandom::new();
        rng.fill(&mut key_config.key).unwrap();
        rng.fill(&mut key_config.nonce).unwrap();
        rng.fill(&mut key_config.ad).unwrap();

        key_config
    }
}

struct OneNonceSequence(Option<aead::Nonce>);

impl OneNonceSequence {
    /// Constructs the sequence allowing `advance()` to be called
    /// `allowed_invocations` times.
    fn new(nonce: aead::Nonce) -> Self {
        Self(Some(nonce))
    }
}

impl aead::NonceSequence for OneNonceSequence {
    fn advance(&mut self) -> core::result::Result<aead::Nonce, ring::error::Unspecified> {
        self.0.take().ok_or(ring::error::Unspecified)
    }
}
fn encrypt_data(mut data: Vec<u8>, aes_key: &[u8], aes_nonce: &[u8], aes_ad: &[u8]) -> Vec<u8> {
    let aead_alg = &aead::AES_256_GCM;

    assert_eq!(aes_key.len(), 32);
    assert_eq!(aes_nonce.len(), 12);
    assert_eq!(aes_ad.len(), 5);

    let ub = UnboundKey::new(aead_alg, aes_key).unwrap();
    let nonce = Nonce::try_assume_unique_for_key(aes_nonce).unwrap();
    let filesequence = OneNonceSequence::new(nonce);

    let mut s_key = aead::SealingKey::new(ub, filesequence);
    let ad = Aad::from(aes_ad);
    let s_result = s_key.seal_in_place_append_tag(ad, &mut data);
    s_result.unwrap();

    data
}

fn gen_and_upload_key(
    info: &MesateeEnclaveInfo,
    key_saving_path: &str,
    key_file_id_saving_path: &str,
) {
    let key_config = AEADKeyConfig::new();
    let key_bytes = serde_json::to_vec(&key_config).unwrap();
    fs::write(key_saving_path, key_bytes).unwrap();
    let mesatee = Mesatee::new(info, USER_ID, USER_TOKEN, *TMS_ADDR, *TDFS_ADDR).unwrap();
    let file_id = mesatee.upload_file(key_saving_path).unwrap();
    fs::write(key_file_id_saving_path, file_id.as_bytes()).unwrap();
}

fn local_encrypt(plaintxt_path: &str, encrypted_data_saving_path: &str, key_path: &str) {
    let key_bytes = fs::read(key_path).unwrap();
    let key_config: AEADKeyConfig = serde_json::from_slice(&key_bytes).unwrap();
    let plaintxt = fs::read(plaintxt_path).unwrap();
    let encrypted_data = encrypt_data(plaintxt, &key_config.key, &key_config.nonce, &key_config.ad);
    fs::write(encrypted_data_saving_path, encrypted_data).unwrap();
}

fn online_decrypt(
    info: &MesateeEnclaveInfo,
    encrypted_data_path: &str,
    key_file_id: &str,
    decrypted_data_saving_path: &str,
) {
    let encrypted_bytes = fs::read(&encrypted_data_path).unwrap();
    let base64_input = base64::encode(&encrypted_bytes);
    let mesatee = Mesatee::new(info, USER_ID, USER_TOKEN, *TMS_ADDR, *TDFS_ADDR).unwrap();
    let task = mesatee
        .create_task_with_files(FUNCTION_NAME, &[key_file_id])
        .unwrap();
    let result = task.invoke_with_payload(&base64_input).unwrap();
    let output_bytes = base64::decode(&result).unwrap();
    fs::write(decrypted_data_saving_path, &output_bytes).unwrap();
}

fn main() {
    let auditors = vec![
        (
            "../services/auditors/godzilla/godzilla.public.der",
            "../services/auditors/godzilla/godzilla.sign.sha256",
        ),
        (
            "../services/auditors/optimus_prime/optimus_prime.public.der",
            "../services/auditors/optimus_prime/optimus_prime.sign.sha256",
        ),
        (
            "../services/auditors/albus_dumbledore/albus_dumbledore.public.der",
            "../services/auditors/albus_dumbledore/albus_dumbledore.sign.sha256",
        ),
    ];
    let enclave_info_file_path = "../services/enclave_info.txt";

    let mesatee_enclave_info = MesateeEnclaveInfo::load(auditors, enclave_info_file_path).unwrap();

    let args_string: Vec<String> = env::args().collect();
    let args: Vec<&str> = args_string.iter().map(|s| s.as_str()).collect();
    if args.len() < 2 {
        print_usage();
        return;
    }
    let action = args[1];
    match action {
        "gen_and_upload_key" => {
            if args.len() != 4 {
                print_usage();
                return;
            }
            gen_and_upload_key(&mesatee_enclave_info, args[2], args[3]);
        }
        "local_encrypt" => {
            if args.len() != 5 {
                print_usage();
                return;
            }
            local_encrypt(args[2], args[3], args[4]);
        }
        "online_decrypt" => {
            if args.len() != 5 {
                print_usage();
                return;
            }
            online_decrypt(&mesatee_enclave_info, args[2], args[3], args[4]);
        }
        _ => {
            print_usage();
        }
    }
}
