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
use std::path::PathBuf;

#[derive(Debug)]
pub struct MesaTEEServiceGenerator;

/// How to use prost. See kms as an example.
/// 1. Define rpc messages with protobuf 2/3 syntax. protobuf 2 is recommended because we can avoid unneccessary option.
/// 2. Define services. Prost will generate corresponding sevices and clients.
/// 3. Include ```prost_build_generator.rs``` and modify ```main function``` in the ```build.rs``` of the target library.  
/// 4. Todo: add support for automatic authentication

impl MesaTEEServiceGenerator {
    fn generate_structure(&mut self, service: &prost_build::Service, buf: &mut String) {
        let request_name = format!("{}{}", &service.proto_name, "Request");
        let response_name = format!("{}{}", &service.proto_name, "Response");
        // Generate request enum structure
        buf.push_str(
            "#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]\n",
        );
        buf.push_str("#[serde(tag = \"type\")]\n");
        buf.push_str(&format!("pub enum {} {{\n", &request_name));
        for method in &service.methods {
            buf.push_str(&format!(
                "    {}({}),\n",
                method.proto_name, method.input_type
            ));
        }
        buf.push_str(&format!("}}\n"));

        // Generate response enum structure
        buf.push_str(
            "#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]\n",
        );
        buf.push_str("#[serde(tag = \"type\")]\n");
        buf.push_str(&format!("pub enum {} {{\n", &response_name));
        for method in &service.methods {
            buf.push_str(&format!(
                "    {}({}),\n",
                method.proto_name, method.output_type
            ));
        }
        buf.push_str(&format!("}}\n"));
    }

    fn generate_service(&mut self, service: &prost_build::Service, buf: &mut String) {
        let request_name = format!("{}{}", &service.proto_name, "Request");
        let response_name = format!("{}{}", &service.proto_name, "Response");
        let service_name = format!("{}{}", &service.proto_name, "Service");
        // Genreate trait
        buf.push_str(&format!("pub trait {} {{\n", &service_name));
        for method in &service.methods {
            buf.push_str(&format!(
                "    fn {}(req: {}) -> mesatee_core::Result<{}>;\n",
                method.name, method.input_type, method.output_type
            ));
        }
        // Generate dispatch
        buf.push_str(&format!(
            "    fn dispatch(&self, req: {}) -> mesatee_core::Result<{}> {{\n",
            &request_name, &response_name
        ));

        // authentication
        let mut need_authentication: bool = false;
        for comment in service.comments.leading.iter() {
            if comment.contains("@need_authentication") {
                need_authentication = true;
            }
        }
        if need_authentication {
            buf.push_str("        let authenticated = match req {\n");
            for method in &service.methods {
                buf.push_str(&format!(
                    "            {}::{}(ref req) => req.creds.auth(),\n",
                    &request_name, &method.proto_name
                ));
            }
            buf.push_str("        };\n");
            buf.push_str(
                r#"
        if !authenticated {
            return Err(mesatee_core::Error::from(mesatee_core::ErrorKind::PermissionDenied));
        }
"#,
            )
        }

        // dispatch request
        buf.push_str("        match req {\n");
        for method in &service.methods {
            buf.push_str(&format!(
                "            {}::{}(req) => Self::{}(req).map({}::{}),\n",
                &request_name, &method.proto_name, method.name, &response_name, &method.proto_name
            ));
        }
        buf.push_str("        }\n");
        buf.push_str("    }\n");
        buf.push_str("}\n");
    }

    fn generate_client(&mut self, service: &prost_build::Service, buf: &mut String) {
        let request_name = format!("{}{}", &service.proto_name, "Request");
        let response_name = format!("{}{}", &service.proto_name, "Response");
        let client_name = format!("{}{}", &service.proto_name, "Client");
        buf.push_str(&format!("pub struct {} {{\n", &client_name));
        buf.push_str(&format!(
            "    channel: mesatee_core::rpc::channel::SgxTrustedChannel<{}, {}>,\n",
            request_name, response_name
        ));
        buf.push_str("}\n");

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
        buf.push_str("}\n");
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
