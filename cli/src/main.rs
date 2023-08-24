// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use http::Uri;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use teaclave_attestation::report::AttestationReport;

use teaclave_crypto::{AesGcm128Key, AesGcm256Key, TeaclaveFile128Key};

const FILE_AUTH_TAG_LENGTH: usize = 16;
type CMac = [u8; FILE_AUTH_TAG_LENGTH];
type KeyVec = Vec<u8>; // Need define a type to use parse derive macro

fn decode_hex(src: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(src)
}

#[derive(Debug, StructOpt)]
struct EncryptDecryptOpt {
    /// Crypto algorithm, supported algorithms are "aes-gcm-128", "aes-gcm-256",
    /// "teaclave-file-128".
    #[structopt(short, long)]
    algorithm: String,

    /// Key in the hex format.
    #[structopt(short, long, parse(try_from_str = decode_hex))]
    key: KeyVec,

    /// IV for AES keys in the hex format.
    #[structopt(long, parse(try_from_str = decode_hex))]
    iv: Option<KeyVec>,

    /// Path of input file.
    #[structopt(short, long = "input-file")]
    input_file: PathBuf,

    /// Path of output file.
    #[structopt(short, long = "output-file")]
    output_file: PathBuf,

    /// Flag to print out CMAC.
    #[structopt(short = "c", long = "print-cmac")]
    print_cmac: bool,
}

#[derive(Debug, StructOpt)]
struct VerifyOpt {
    /// Path of enclave info
    #[structopt(short, long = "enclave-info")]
    enclave_info: PathBuf,

    /// Path of signatures
    #[structopt(required = true, short, long)]
    signatures: Vec<PathBuf>,

    /// Path of auditor's public key
    #[structopt(required = true, short, long = "public-keys")]
    public_keys: Vec<PathBuf>,
}

#[derive(Debug, StructOpt)]
struct AttestOpt {
    /// Address of the remote service
    #[structopt(short, long)]
    address: String,

    /// CA cert of attestation service for verifying the attestation report
    #[structopt(short = "c", long)]
    as_ca_cert: PathBuf,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Encrypt file
    #[structopt(name = "encrypt")]
    Encrypt(EncryptDecryptOpt),

    /// Decrypt file
    #[structopt(name = "decrypt")]
    Decrypt(EncryptDecryptOpt),

    /// Verify signatures of enclave info with auditors' public keys
    #[structopt(name = "verify")]
    Verify(VerifyOpt),

    /// Display the attestation report of remote Teaclave services
    #[structopt(name = "attest")]
    Attest(AttestOpt),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "teaclave_cli", about = "Teaclave command line tool.")]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

fn decrypt(opt: EncryptDecryptOpt) -> Result<CMac> {
    let key = opt.key;
    let mut cmac: CMac = [0u8; FILE_AUTH_TAG_LENGTH];
    match opt.algorithm.as_str() {
        AesGcm128Key::SCHEMA => {
            let iv = opt.iv.expect("IV is required.");
            let key = AesGcm128Key::new(&key, &iv)?;
            let mut content = fs::read(opt.input_file)?;
            let res = key.decrypt(&mut content)?;
            cmac.copy_from_slice(&res);
            fs::write(opt.output_file, content)?;
        }
        AesGcm256Key::SCHEMA => {
            let iv = opt.iv.expect("IV is required.");
            let key = AesGcm256Key::new(&key, &iv)?;
            let mut content = fs::read(opt.input_file)?;
            let res = key.decrypt(&mut content)?;
            cmac.copy_from_slice(&res);
            fs::write(opt.output_file, content)?;
        }
        TeaclaveFile128Key::SCHEMA => {
            let key = TeaclaveFile128Key::new(&key)?;
            let mut output_file = fs::File::create(opt.output_file)?;
            let res = key.decrypt(opt.input_file, &mut output_file)?;
            cmac.copy_from_slice(&res);
        }
        _ => bail!("Invalid crypto algorithm"),
    }

    Ok(cmac)
}

fn encrypt(opt: EncryptDecryptOpt) -> Result<CMac> {
    let key = opt.key;
    let mut cmac: CMac = [0u8; FILE_AUTH_TAG_LENGTH];
    match opt.algorithm.as_str() {
        AesGcm128Key::SCHEMA => {
            let iv = opt.iv.expect("IV is required.");
            let key = AesGcm128Key::new(&key, &iv)?;
            let mut content = fs::read(opt.input_file)?;
            let res = key.encrypt(&mut content)?;
            cmac.copy_from_slice(&res);
            fs::write(opt.output_file, content)?;
        }
        AesGcm256Key::SCHEMA => {
            let iv = opt.iv.expect("IV is required.");
            let key = AesGcm256Key::new(&key, &iv)?;
            let mut content = fs::read(opt.input_file)?;
            let res = key.encrypt(&mut content)?;
            cmac.copy_from_slice(&res);
            fs::write(opt.output_file, content)?;
        }
        TeaclaveFile128Key::SCHEMA => {
            let key = TeaclaveFile128Key::new(&key)?;
            let content = fs::File::open(opt.input_file)?;
            let res = key.encrypt(opt.output_file, content)?;
            cmac.copy_from_slice(&res);
        }
        _ => bail!("Invalid crypto algorithm"),
    }

    Ok(cmac)
}

fn verify(opt: VerifyOpt) -> Result<bool> {
    let enclave_info = fs::read(opt.enclave_info)?;
    let mut public_keys = Vec::new();
    let mut signatures = Vec::new();
    for p in opt.public_keys {
        let content = fs::read(p)?;
        let pem = pem::parse(content).expect("Expect a valid PEM file");
        public_keys.push(pem.contents);
    }

    for s in opt.signatures {
        signatures.push(fs::read(s)?);
    }

    Ok(teaclave_types::EnclaveInfo::verify(
        &enclave_info,
        &public_keys,
        &signatures,
    ))
}

struct TeaclaveServerCertVerifier {
    pub root_ca: Vec<u8>,
}

impl TeaclaveServerCertVerifier {
    pub fn new(root_ca: &[u8]) -> Self {
        Self {
            root_ca: root_ca.to_vec(),
        }
    }

    fn display_attestation_report(&self, certs: &[rustls::Certificate]) -> bool {
        match AttestationReport::from_cert(certs, &self.root_ca) {
            Ok(report) => println!("{}", report),
            Err(e) => println!("{:?}", e),
        }
        true
    }
}

impl rustls::client::ServerCertVerifier for TeaclaveServerCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp: &[u8],
        _now: std::time::SystemTime,
    ) -> std::result::Result<rustls::client::ServerCertVerified, rustls::Error> {
        // This call automatically verifies certificate signature
        if self.display_attestation_report(&[end_entity.to_owned()]) {
            Ok(rustls::client::ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnhandledCriticalExtension,
            ))
        }
    }
}

fn attest(opt: AttestOpt) -> Result<()> {
    let uri = opt.address.parse::<Uri>()?;
    let hostname = uri.host().ok_or_else(|| anyhow!("Invalid hostname."))?;
    let mut stream = std::net::TcpStream::connect(opt.address)?;
    let content = fs::read(opt.as_ca_cert)?;
    let pem = pem::parse(content)?;
    let verifier = Arc::new(TeaclaveServerCertVerifier::new(&pem.contents));
    let config = rustls::ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS12])
        .unwrap()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth();
    let mut session =
        rustls::client::ClientConnection::new(Arc::new(config), hostname.try_into()?)?;
    let mut tls_stream = rustls::Stream::new(&mut session, &mut stream);
    tls_stream.write_all(&[0]).unwrap();

    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Opt::from_args();
    match args.command {
        Command::Decrypt(opt) => {
            let flag = opt.print_cmac;
            let cmac = decrypt(opt)?;
            if flag {
                let cmac_string = hex::encode(cmac);
                println!("{}", cmac_string);
            }
        }
        Command::Encrypt(opt) => {
            let flag = opt.print_cmac;
            let cmac = encrypt(opt)?;
            if flag {
                let cmac_string = hex::encode(cmac);
                println!("{}", cmac_string);
            }
        }
        Command::Verify(opt) => match verify(opt) {
            Ok(false) | Err(_) => bail!("Failed to verify signatures."),
            Ok(true) => {
                println!("Verify successfully.");
                return Ok(());
            }
        },
        Command::Attest(opt) => attest(opt)?,
    };

    Ok(())
}
