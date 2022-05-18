use std::prelude::v1::*;

use std::{collections::HashMap, convert::TryFrom, convert::From, env, sync::SgxMutex as Mutex};
use std::iter::FromIterator;
use std::convert::Into;

use tvm_graph_rt::{Graph, GraphExecutor, SystemLibModule, Tensor as TVMTensor};
use lazy_static::lazy_static;
use ndarray::Array;
//use image::{FilterType, GenericImageView};
//use serde::{Serialize, Deserialize};

lazy_static! {
    static ref SYSLIB: SystemLibModule = SystemLibModule::default();
    static ref GRAPH_EXECUTOR: Mutex<GraphExecutor<'static, 'static>> = {
        let graph = Graph::try_from(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/lib/graph.json"
        )))
        .unwrap();
        let params_bytes =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/lib/graph.params"));
        let params = tvm_graph_rt::load_param_dict(params_bytes)
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, v.to_owned()))
            .collect::<HashMap<String, TVMTensor<'static>>>();

        let mut exec = GraphExecutor::new(graph, &*SYSLIB).unwrap();
        exec.load_params(params);

        Mutex::new(exec)
    };
}

const IMG_HEIGHT: usize = 224;
const IMG_WIDTH: usize = 224;

fn data_preprocess(normalized_pixels: Vec<f32>) -> TVMTensor<'static> {
    // (H,W,C) -> (C,H,W)
    let arr = Array::from_shape_vec((IMG_HEIGHT, IMG_WIDTH, 3), normalized_pixels).unwrap();
    let arr = arr.permuted_axes([2, 0, 1]);
    let arr = Array::from_iter(arr.into_iter().copied().map(|v| v));
    TVMTensor::from(arr)
}

pub fn run_inference(pixels: Vec<f32>) -> Vec<f32> {
    println!("[+] run_inference");
    let input = data_preprocess(pixels);
    println!("[+] processed_input");
    GRAPH_EXECUTOR.lock().unwrap().set_input("input_1", input);
    println!("[+] set input");
    GRAPH_EXECUTOR.lock().unwrap().run();
    println!("[+] finish run");
    GRAPH_EXECUTOR
        .lock()
        .unwrap()
        .get_output(0)
        .unwrap().to_vec()
}