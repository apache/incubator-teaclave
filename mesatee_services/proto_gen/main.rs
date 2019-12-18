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

use prost_build;
use std::path;
use structopt::StructOpt;

#[derive(Debug)]
pub struct MesaTEEServiceGenerator;

/// How to use prost. See kms as an example.
/// 1. Define rpc messages with protobuf 2/3 syntax. protobuf 2 is recommended
/// because we can avoid unneccessary option.
/// 2. Define services. Prost will generate corresponding sevices and clients.
/// 3. Include ```${OUT_DIR}/kms_proto.rs``` and provide serializer and
/// deserializer if needed..
/// 4. Todo: add support for automatic authentication
const LINE_ENDING: &'static str = "\n";
impl MesaTEEServiceGenerator {
    fn generate_structure(&mut self, service: &prost_build::Service, buf: &mut String) {
        let request_name = format!("{}{}", &service.proto_name, "Request");
        let response_name = format!("{}{}", &service.proto_name, "Response");
        // Generate request enum structure
        buf.push_str("#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]");
        buf.push_str(LINE_ENDING);
        buf.push_str(r#"#[serde(tag = "type")]"#);

        buf.push_str(&format!("pub enum {} {{", &request_name));
        buf.push_str(LINE_ENDING);
        for method in &service.methods {
            buf.push_str(&format!(
                "    {}({}),",
                method.proto_name, method.input_type
            ));
            buf.push_str(LINE_ENDING);
        }
        buf.push_str("}");
        buf.push_str(LINE_ENDING);

        // Generate response enum structure
        buf.push_str("#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]");
        buf.push_str(LINE_ENDING);
        buf.push_str(r#"#[serde(tag = "type")]"#);
        buf.push_str(LINE_ENDING);
        buf.push_str(&format!("pub enum {} {{", &response_name));
        buf.push_str(LINE_ENDING);
        for method in &service.methods {
            buf.push_str(&format!(
                "    {}({}),",
                method.proto_name, method.output_type
            ));
            buf.push_str(LINE_ENDING);
        }
        buf.push_str("}");
        buf.push_str(LINE_ENDING);
    }

    fn generate_service(&mut self, service: &prost_build::Service, buf: &mut String) {
        let request_name = format!("{}{}", &service.proto_name, "Request");
        let response_name = format!("{}{}", &service.proto_name, "Response");
        let service_name = format!("{}{}", &service.proto_name, "Service");
        // Genreate trait
        buf.push_str(&format!("pub trait {} {{", &service_name));
        buf.push_str(LINE_ENDING);
        for method in &service.methods {
            buf.push_str(&format!(
                "    fn {}(req: {}) -> mesatee_core::Result<{}>;",
                method.name, method.input_type, method.output_type
            ));
            buf.push_str(LINE_ENDING);
        }
        // Generate dispatch
        buf.push_str(&format!(
            "    fn dispatch(&self, req: {}) -> mesatee_core::Result<{}> {{",
            &request_name, &response_name
        ));
        buf.push_str(LINE_ENDING);

        // authentication
        let mut need_authentication: bool = false;
        for comment in service.comments.leading.iter() {
            if comment.contains("@need_authentication") {
                need_authentication = true;
            }
        }
        if need_authentication {
            buf.push_str("        let authenticated = match req {");
            buf.push_str(LINE_ENDING);
            for method in &service.methods {
                buf.push_str(&format!(
                    "            {}::{}(ref req) => req.creds.auth(),",
                    &request_name, &method.proto_name
                ));
                buf.push_str(LINE_ENDING);
            }
            buf.push_str("        };");
            buf.push_str(
                r#"
        if !authenticated {
            return Err(mesatee_core::Error::from(mesatee_core::ErrorKind::PermissionDenied));
        }
"#,
            )
        }

        // dispatch request
        buf.push_str("        match req {");
        buf.push_str(LINE_ENDING);
        for method in &service.methods {
            buf.push_str(&format!(
                "            {}::{}(req) => Self::{}(req).map({}::{}),",
                &request_name, &method.proto_name, method.name, &response_name, &method.proto_name
            ));
        }
        buf.push_str("        }");
        buf.push_str(LINE_ENDING);
        buf.push_str("    }");
        buf.push_str(LINE_ENDING);
        buf.push_str("}");
        buf.push_str(LINE_ENDING);
    }

    fn generate_client(&mut self, service: &prost_build::Service, buf: &mut String) {
        let request_name = format!("{}{}", &service.proto_name, "Request");
        let response_name = format!("{}{}", &service.proto_name, "Response");
        let client_name = format!("{}{}", &service.proto_name, "Client");
        buf.push_str(&format!("pub struct {} {{", &client_name));
        buf.push_str(LINE_ENDING);
        buf.push_str(&format!(
            "    channel: mesatee_core::rpc::channel::SgxTrustedChannel<{}, {}>,",
            request_name, response_name
        ));
        buf.push_str(LINE_ENDING);
        buf.push_str("}");
        buf.push_str(LINE_ENDING);

        // impl new
        buf.push_str(&format!(
            r#"
impl {} {{
    pub fn new(target: mesatee_core::config::TargetDesc) -> mesatee_core::Result<Self> {{
        let addr = target.addr;
        let channel = match target.desc {{
            mesatee_core::config::OutboundDesc::Sgx(enclave_addr) => {{
                mesatee_core::rpc::channel::SgxTrustedChannel::<{}, {}>::new(addr, enclave_addr)?
            }}
        }};
        Ok({} {{ channel }})
    }}
}}
"#,
            client_name, request_name, response_name, client_name
        ));

        // impl operation
        buf.push_str(&format!("impl {} {{", client_name));
        for method in &service.methods {
            buf.push_str(&format!(
                r#"
    pub fn {}(&mut self, req: {}) -> mesatee_core::Result<{}> {{
        let req = {}::{}(req);
        let resp = self.channel.invoke(req)?;
        match resp {{
            {}::{}(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }}
    }}
"#,
                &method.name,
                &method.input_type,
                &method.output_type,
                &request_name,
                &method.proto_name,
                &response_name,
                &method.proto_name
            ));
        }
        buf.push_str("}");
        buf.push_str(LINE_ENDING);
    }
}

impl prost_build::ServiceGenerator for MesaTEEServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        self.generate_structure(&service, buf);
        self.generate_service(&service, buf);
        self.generate_client(&service, buf);
    }
}

pub fn get_default_config() -> prost_build::Config {
    let mut config = prost_build::Config::new();
    config.service_generator(Box::new(MesaTEEServiceGenerator));
    config.type_attribute(
        ".",
        "#[derive(serde_derive::Serialize, serde_derive::Deserialize)]",
    );
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
