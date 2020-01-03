#![allow(clippy::all)]

use anyhow::anyhow;
use anyhow::Result;
use teaclave_rpc::channel;

include!(concat!(env!("OUT_DIR"), "/teaclave_frontend_proto.rs"));
