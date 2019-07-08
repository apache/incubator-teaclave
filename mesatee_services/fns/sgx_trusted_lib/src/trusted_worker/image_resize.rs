// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::trait_defs::{WorkerHelper, WorkerInput};
use image::{FilterType, ImageOutputFormat};
use mesatee_core::{Error, ErrorKind, Result};
use serde_derive::Deserialize;
use serde_json;

//INPUT
#[derive(Deserialize)]
pub(crate) struct ImageResizePayload {
    nwidth: u32,
    nheight: u32,
    filter_type: String, //"Nearest", "Triangle", "CatmullRom", "Gaussian", "Lanczos3"
    output_format: String, //"PNG", "JPEG", , "GIF", "ICO", "BMP"
    base64_image: String,
}
//OUTPUT: image bytes encoded with base64

pub(crate) fn process(_helper: &mut WorkerHelper, input: WorkerInput) -> Result<String> {
    let payload = input
        .payload
        .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;

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
    let input_image = image::load_from_memory(&image_bytes)
        .or_else(|_| Err(Error::from(ErrorKind::InvalidInputError)))?;
    let new_image =
        input_image.resize_exact(resize_payload.nwidth, resize_payload.nheight, filter_type);
    let mut output_bytes: Vec<u8> = Vec::new();
    new_image
        .write_to(&mut output_bytes, output_format)
        .map_err(|_| Error::from(ErrorKind::OutputGenerationError))?;
    let output_base64 = base64::encode(&output_bytes);
    Ok(output_base64)
}
