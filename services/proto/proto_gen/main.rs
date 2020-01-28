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

use askama;
use askama::Template;
use prost_build;
use std::path;
use structopt::StructOpt;

#[derive(Debug)]
pub struct MesaTEEServiceGenerator;

#[derive(Template)]
#[template(path = "proto.j2")]
struct ProtoTemplate<'a> {
    service: &'a prost_build::Service,
    proto_impl_mod_name: &'a str,
}

impl MesaTEEServiceGenerator {
    fn generate_from_template(&mut self, service: &prost_build::Service, buf: &mut String) {
        let name_len = service.package.len();
        let proto_template = ProtoTemplate {
            service,
            proto_impl_mod_name: &service.package[0..name_len - "_proto".len()],
        };
        buf.push_str(&proto_template.render().unwrap());
    }
}

impl prost_build::ServiceGenerator for MesaTEEServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        self.generate_from_template(&service, buf);
    }
}

pub fn get_default_config() -> prost_build::Config {
    let mut config = prost_build::Config::new();
    config.service_generator(Box::new(MesaTEEServiceGenerator));
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config
}

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "p", required = true)]
    /// Paths to .proto files to compile. Any transitively imported .proto files
    /// are automatically be included.
    protos: Vec<path::PathBuf>,

    #[structopt(short = "i", required = true)]
    /// Paths to directories in which to search for imports. Directories are
    /// searched in order. The .proto files passed in protos must be found in
    /// one of the provided include directories.
    includes: Vec<path::PathBuf>,

    #[structopt(short = "d", required = true)]
    /// Configures the output directory where generated Rust files will be written.
    out_dir: path::PathBuf,
}

fn main() {
    let args = Cli::from_args();
    let mut config = get_default_config();
    config.out_dir(args.out_dir);
    config.compile_protos(&args.protos, &args.includes).unwrap();
}
