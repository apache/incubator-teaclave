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

use anyhow::bail;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

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

    /// Key in hex format
    #[structopt(short, long, parse(try_from_str = decode_hex))]
    key: KeyVec,

    /// IV for AES keys in hex format
    #[structopt(long, parse(try_from_str = decode_hex))]
    iv: Option<KeyVec>,

    /// Path of input file
    #[structopt(short, long = "input-file")]
    input_file: PathBuf,

    /// Path of output file
    #[structopt(short, long = "output-file")]
    output_file: PathBuf,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Encrypt file
    #[structopt(name = "encrypt")]
    Encrypt(EncryptDecryptOpt),

    /// Decrypt file
    #[structopt(name = "decrypt")]
    Decrypt(EncryptDecryptOpt),
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
            let mut content = vec![];
            let res = key.decrypt(opt.input_file, &mut content)?;
            cmac.copy_from_slice(&res);
            fs::write(opt.output_file, content)?;
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
            let content = fs::read(opt.input_file)?;
            let res = key.encrypt(opt.output_file, &content)?;
            cmac.copy_from_slice(&res);
        }
        _ => bail!("Invalid crypto algorithm"),
    }

    Ok(cmac)
}

fn main() -> Result<()> {
    let args = Opt::from_args();
    let cmac = match args.command {
        Command::Decrypt(opt) => decrypt(opt)?,
        Command::Encrypt(opt) => encrypt(opt)?,
    };
    let cmac_string = hex::encode(cmac);
    println!("{}", cmac_string);
    Ok(())
}
