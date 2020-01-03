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
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::worker::{FunctionType, Worker, WorkerContext};
use image::{FilterType, ImageOutputFormat};
use mesatee_core::{Error, ErrorKind, Result};
use serde_derive::Deserialize;
use serde_json;

#[derive(Deserialize)]
struct ImageResizePayload {
    nwidth: u32,
    nheight: u32,
    filter_type: String, //"Nearest", "Triangle", "CatmullRom", "Gaussian", "Lanczos3"
    output_format: String, //"PNG", "JPEG", , "GIF", "ICO", "BMP"
    base64_image: String,
}

pub struct ImageResizeWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<ImageResizeWorkerInput>,
}

struct ImageResizeWorkerInput {
    nwidth: u32,
    nheight: u32,
    filter_type: FilterType, //"Nearest", "Triangle", "CatmullRom", "Gaussian", "Lanczos3"
    output_format: ImageOutputFormat, //"PNG", "JPEG", , "GIF", "ICO", "BMP"
    image_bytes: Vec<u8>,
}

impl ImageResizeWorker {
    pub fn new() -> Self {
        ImageResizeWorker {
            worker_id: 0,
            func_name: "image_resize".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}

impl Worker for ImageResizeWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn function_type(&self) -> FunctionType {
        self.func_type
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(
        &mut self,
        dynamic_input: Option<String>,
        _file_ids: Vec<String>,
    ) -> Result<()> {
        let payload = dynamic_input.ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;

        let resize_payload: ImageResizePayload = serde_json::from_str(&payload)
            .or_else(|_| Err(Error::from(ErrorKind::InvalidInputError)))?;

        let filter_type = match resize_payload.filter_type.as_str() {
            "Triangle" => FilterType::Triangle,
            "CatmullRom" => FilterType::CatmullRom,
            "Gaussian" => FilterType::Gaussian,
            "Lanczos3" => FilterType::Lanczos3,
            _ => FilterType::Nearest,
        };

        let output_format = match resize_payload.output_format.as_str() {
            "PNG" => ImageOutputFormat::PNG,
            "GIF" => ImageOutputFormat::GIF,
            "ICO" => ImageOutputFormat::ICO,
            "BMP" => ImageOutputFormat::BMP,
            _ => ImageOutputFormat::JPEG(75),
        };

        let image_bytes: Vec<u8> = base64::decode(&resize_payload.base64_image)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

        self.input = Some(ImageResizeWorkerInput {
            nwidth: resize_payload.nwidth,
            nheight: resize_payload.nheight,
            filter_type,
            output_format,
            image_bytes,
        });
        Ok(())
    }

    fn execute(&mut self, _context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let input_image = image::load_from_memory(&input.image_bytes)
            .or_else(|_| Err(Error::from(ErrorKind::InvalidInputError)))?;
        let new_image = input_image.resize_exact(input.nwidth, input.nheight, input.filter_type);
        let mut output_bytes: Vec<u8> = Vec::new();
        new_image
            .write_to(&mut output_bytes, input.output_format)
            .map_err(|_| Error::from(ErrorKind::OutputGenerationError))?;
        let output_base64 = base64::encode(&output_bytes);
        Ok(output_base64)
    }
}
