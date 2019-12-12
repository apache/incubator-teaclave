use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct BuildConfigToml {
    sp_root_ca_cert: ConfigSource,
    ias_root_ca_cert: ConfigSource,
    auditor_public_keys: Vec<ConfigSource>,
    rpc_max_message_size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
enum ConfigSource {
    Path(PathBuf),
}

fn display_config_source(config: &ConfigSource) -> String {
    match config {
        ConfigSource::Path(p) => {
            let content = &fs::read(p).expect(&format!("Failed to read file: {}", p.display()));
            let mut output = String::new();
            output.push_str("&[");
            for b in content {
                output.push_str(&format!("{}, ", b));
            }
            output.push_str("]");

            output
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("Please specify the path of build config toml and output path.");
    }
    let contents = fs::read_to_string(&args[1]).expect("Something went wrong reading the file");
    let config: BuildConfigToml = toml::from_str(&contents).expect("Failed to parse the config.");

    let sp_root_ca_cert = display_config_source(&config.sp_root_ca_cert);
    let ias_root_ca_cert = display_config_source(&config.ias_root_ca_cert);

    let mut auditor_public_keys = String::new();
    auditor_public_keys.push_str("&[");
    for key in &config.auditor_public_keys {
        let auditor_pulic_key = display_config_source(key);
        auditor_public_keys.push_str(&format!("{}, ", auditor_pulic_key));
    }
    auditor_public_keys.push_str("]");

    let mut build_config_generated = String::new();
    build_config_generated.push_str(&format!(
        r#"
    #[derive(Debug)]
    pub struct BuildConfig<'a> {{
        pub sp_root_ca_cert: &'a [u8],
        pub ias_root_ca_cert: &'a [u8],
        pub auditor_public_keys: &'a [&'a [u8];{}],
        pub rpc_max_message_size: u64,
    }}

    pub static BUILD_CONFIG: BuildConfig<'static> = BuildConfig {{
        sp_root_ca_cert: {},
        ias_root_ca_cert: {},
        auditor_public_keys: {},
        rpc_max_message_size: {},
    }};"#,
        config.auditor_public_keys.len(),
        sp_root_ca_cert,
        ias_root_ca_cert,
        auditor_public_keys,
        config.rpc_max_message_size
    ));

    let dest_path = Path::new(&args[2]);
    let mut f =
        File::create(&dest_path).expect(&format!("Failed to create file: {}", dest_path.display()));
    f.write_all(build_config_generated.as_bytes())
        .expect(&format!("Failed to write file: {}", dest_path.display()));
}
