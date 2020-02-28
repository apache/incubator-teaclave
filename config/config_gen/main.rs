use askama;
use askama::Template;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Serialize, Deserialize)]
struct BuildConfigToml {
    as_root_ca_cert: ConfigSource,
    auditor_public_keys: Vec<ConfigSource>,
    rpc_max_message_size: u64,
    attestation_validity_secs: u64,
    inbound: Inbound,
}

#[derive(Serialize, Deserialize)]
struct Inbound {
    access_control: Vec<String>,
    authentication: Vec<String>,
    management: Vec<String>,
    storage: Vec<String>,
    scheduler: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
enum ConfigSource {
    Path(PathBuf),
}

fn display_config_source(config: &ConfigSource) -> String {
    match config {
        ConfigSource::Path(p) => match p.extension().and_then(std::ffi::OsStr::to_str) {
            Some("pem") => {
                let content = &fs::read(p).expect(&format!("Failed to read file: {}", p.display()));
                let pem = pem::parse(content).expect("Cannot parse PEM file");
                format!("{:?}", pem.contents)
            }
            _ => {
                let content = &fs::read(p).expect(&format!("Failed to read file: {}", p.display()));
                format!("{:?}", content)
            }
        },
    }
}

#[derive(Template)]
#[template(path = "config.j2")]
struct ConfigTemplate {
    as_root_ca_cert: String,
    auditor_public_keys: Vec<String>,
    rpc_max_message_size: u64,
    attestation_validity_secs: u64,
    inbound: Inbound,
}

fn generate_build_config(toml: &Path, out: &Path) {
    let contents = fs::read_to_string(toml).expect("Something went wrong reading the file");
    let config: BuildConfigToml = toml::from_str(&contents).expect("Failed to parse the config.");

    let as_root_ca_cert = display_config_source(&config.as_root_ca_cert);

    let mut auditor_public_keys: Vec<String> = vec![];
    for key in &config.auditor_public_keys {
        let auditor_pulic_key = display_config_source(key);
        auditor_public_keys.push(auditor_pulic_key);
    }
    let config_template = ConfigTemplate {
        as_root_ca_cert,
        auditor_public_keys,
        rpc_max_message_size: config.rpc_max_message_size,
        attestation_validity_secs: config.attestation_validity_secs,
        inbound: config.inbound,
    };
    let mut f = File::create(out).expect(&format!("Failed to create file: {}", out.display()));
    f.write_all(&config_template.render().unwrap().as_bytes())
        .expect(&format!("Failed to write file: {}", out.display()));
}

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "t", required = true)]
    /// Path to the config file in toml.
    toml_path: path::PathBuf,

    #[structopt(short = "o", required = true)]
    /// Configures the output path where generated Rust file will be written.
    out_path: path::PathBuf,
}

fn main() {
    let args = Cli::from_args();
    generate_build_config(&args.toml_path, &args.out_path);
}
