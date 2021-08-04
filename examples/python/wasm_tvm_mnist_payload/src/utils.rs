/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use super::types::*;
use image::imageops::FilterType;
use ndarray::Array;
use std::io::{BufReader, Read};
use teaclave_context::*;

const IMG_HEIGHT: usize = 28;
const IMG_WIDTH: usize = 28;

pub fn load_input(in_filename: &str) -> Tensor {
    let img_file = TeaclaveContextFile::open_input(in_filename).unwrap();

    let mut reader = BufReader::new(img_file);
    let mut buf = vec![];
    let _ = reader.read_to_end(&mut buf);

    let img = image::load_from_memory(&buf).unwrap();

    data_preprocess(img)
}

pub fn handle_output(out_tensor: Tensor) -> i32 {
    let out_vec = out_tensor.to_vec::<f32>();

    let mut max_idx = 0;
    for i in 0..out_vec.len() {
        if out_vec[i] > out_vec[max_idx] {
            max_idx = i;
        }
    }

    max_idx as _
}

fn data_preprocess(img: image::DynamicImage) -> Tensor {
    let img = img
        .resize_exact(IMG_HEIGHT as u32, IMG_WIDTH as u32, FilterType::Nearest)
        .grayscale();

    let pixels = img.raw_pixels();

    let mut averaged = vec![];
    for p in pixels {
        averaged.push((p as f32) / 255.0);
    }

    let arr = Array::from_shape_vec((IMG_HEIGHT, IMG_WIDTH, 1), averaged).unwrap();
    let arr = Array::from_iter(arr.into_iter().copied().map(|v| v));
    Tensor::from(arr)
}
