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

use askama::Template;
use std::path;
use structopt::StructOpt;

#[derive(Debug)]
pub struct MesaTEEServiceGenerator;

#[derive(Template)]
#[template(path = "proto.j2")]
struct ProtoTemplate {
    service: Service,
}

struct Method {
    name: String,
    proto_name: String,
    input_type: String,
    impl_input_type: String,
    output_type: String,
    impl_output_type: String,
}

struct Service {
    proto_name: String,
    methods: Vec<Method>,
}

impl Service {
    fn from_prost(prost_service: &prost_build::Service) -> Self {
        fn convert_to_impl_type(current_package_name: &str, proto_type: &str) -> String {
            format!(
                "crate::{}::{}",
                current_package_name,
                proto_type.rsplitn(2, "::").collect::<Vec<&str>>()[0].replacen("_proto", "", 1)
            )
        }
        let mut methods = vec![];
        let package_name = prost_service.package.trim_end_matches("_proto");
        for m in prost_service.methods.iter() {
            let impl_input_type = convert_to_impl_type(&package_name, &m.input_type);
            let impl_output_type = convert_to_impl_type(&package_name, &m.output_type);

            let method = Method {
                name: m.name.clone(),
                proto_name: m.proto_name.clone(),
                input_type: m.input_type.clone(),
                impl_input_type,
                output_type: m.output_type.clone(),
                impl_output_type,
            };
            methods.push(method);
        }
        Self {
            proto_name: prost_service.proto_name.clone(),
            methods,
        }
    }
}

impl MesaTEEServiceGenerator {
    fn generate_from_template(&mut self, service: &prost_build::Service, buf: &mut String) {
        let service = Service::from_prost(service);
        let proto_template = ProtoTemplate { service };
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
