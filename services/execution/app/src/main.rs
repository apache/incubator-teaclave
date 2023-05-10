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

#[cfg(feature = "libos")]
fn main() {
    let env = env_logger::Env::new()
        .filter_or("TEACLAVE_LOG", "RUST_LOG")
        .write_style_or("TEACLAVE_LOG_STYLE", "RUST_LOG_STYLE");
    let env_logger = env_logger::Builder::from_env(env).build();
    teaclave_logger::Builder::new()
        .secondary_logger(env_logger)
        .init();
    // The Absolute path of runtime.config.toml in occlum instance
    let config_path = "runtime.config.toml";
    let config = teaclave_config::RuntimeConfig::from_toml(config_path)
        .expect("Failed to load config file.");
    let result = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()
        .expect("failed to create tokio runtime")
        .block_on(teaclave_execution_service_enclave::start_service(&config));
    if result.is_err() {
        log::error!("app will exit, error {:?}", result);
    }
}

#[cfg(not(feature = "libos"))]
fn main() -> anyhow::Result<()> {
    // Use to import ocall
    pub use teaclave_file_agent::ocall_handle_file_request;
    const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
    teaclave_service_app_utils::launch_teaclave_service(PACKAGE_NAME)
}
