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

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

use protected_fs::ProtectedFile;

#[derive(Debug, StructOpt)]
struct EncryptDecryptOpt {
    /// Crypto algorithm
    #[structopt(short, long)]
    algorithm: String,

    /// Key in hex format
    #[structopt(short, long)]
    key: String,

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

fn decrypt(opt: EncryptDecryptOpt) -> Result<()> {
    use std::io::Read;

    if opt.algorithm != "teaclave_file_128" {
        unimplemented!()
    }

    let key_vec = hex::decode(opt.key)?;
    let mut key = [0u8; 16];
    key.copy_from_slice(&key_vec[..16]);
    let mut file = ProtectedFile::open_ex(opt.input_file, &key)?;
    let mut content = vec![];
    file.read_to_end(&mut content)?;
    fs::write(opt.output_file, content)?;

    Ok(())
}

fn main() -> Result<()> {
    let args = Opt::from_args();
    match args.command {
        Command::Decrypt(opt) => decrypt(opt)?,
        _ => unimplemented!(),
    }

    Ok(())
}
