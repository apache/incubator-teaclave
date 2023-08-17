#![forbid(unsafe_code)]

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use policy_core::error::{PolicyCarryingError, PolicyCarryingResult};
use policy_parser::policy_parser::PolicyParser;

pub mod generator;

use generator::codegen_output;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path of the input file.
    #[arg(short, long)]
    input: String,

    /// The path of the output Rust project generated from `codegen`.
    #[arg(short, long, default_value_t = String::from("./api.rs"))]
    output: String,

    /// Should we create a new Cargo project for the generated code.
    #[arg(short, long, default_value_t = false)]
    project: bool,

    /// If the output exists, should we override it?
    #[arg(long, default_value_t = false)]
    r#override: bool,
}

fn init_logger() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    env_logger::init();
}

fn read_file<P: Into<PathBuf>>(path: P) -> PolicyCarryingResult<String> {
    let path: PathBuf = path.into();

    fs::read_to_string(path).map_err(|e| PolicyCarryingError::FsError(e.to_string()))
}

fn main() {
    init_logger();

    let args = Args::parse();

    if let Err(e) = entry(args) {
        log::error!("failed to generate API from policy:\n{}", e);
    }
}

fn entry(args: Args) -> PolicyCarryingResult<()> {
    let parser = PolicyParser::new();
    let policy_content = read_file(&args.input)?;

    let mut policy = parser
        .parse(&policy_content)
        .map_err(|e| PolicyCarryingError::ParseError(args.input.clone(), e.to_string()))?;
    policy.postprocess();

    let (output_file_name, build_file) = if args.project {
        log::info!("trying to create a Cargo project...");

        if Path::new(&args.output).exists() {
            if !args.r#override {
                return Err(PolicyCarryingError::FsError(format!(
                    "{} already exists",
                    args.output
                )));
            } else {
                fs::remove_dir_all(&args.output).unwrap();
            }
        }

        let output = Command::new("cargo")
            .arg("new")
            .arg(&args.output)
            .arg("--lib")
            .output()
            .expect("failed to execute `cargo`");

        if !output.status.success() {
            log::error!("cargo:\n{}", std::str::from_utf8(&output.stderr).unwrap());

            panic!("failed to execute `cargo`");
        }

        (
            format!("{}/src/lib.rs", args.output),
            format!("{}/src/build.rs", args.output),
        )
    } else {
        (args.output, "build.rs".into())
    };

    codegen_output(policy, output_file_name, build_file)
}
