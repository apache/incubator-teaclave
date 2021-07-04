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

extern crate base64;
extern crate image;
#[cfg(feature = "mesalock_sgx")]
extern crate rustface;

use std::prelude::v1::*;

use std::convert::TryFrom;
use teaclave_types::{FunctionArguments, FunctionRuntime};

#[derive(Default)]
pub struct FaceDetection;

#[derive(serde::Deserialize)]
struct FaceDetectionArguments {
    image: Vec<u8>,
    /// Set the size of the sliding window.
    ///
    /// The minimum size is constrained as no smaller than 20.
    ///
    /// # Panics
    ///
    /// Panics if `wnd_size` is less than 20.
    window_size: Option<u32>,
    /// Set the sliding window step in horizontal and vertical directions.
    ///
    /// The steps should take positive values.
    /// Usually a step of 4 is a reasonable choice.
    ///
    /// # Panics
    ///
    /// Panics if `step_x` or `step_y` is less than or equal to 0.
    slide_window_step_x: Option<u32>,
    slide_window_step_y: Option<u32>,
    /// Set the minimum size of faces to detect.
    ///
    /// The minimum size is constrained as no smaller than 20.
    ///
    /// # Panics
    ///
    /// Panics if `min_face_size` is less than 20.
    min_face_size: Option<u32>,
    /// Set the maximum size of faces to detect.
    ///
    /// The maximum face size actually used is computed as the minimum among:
    /// user specified size, image width, image height.
    max_face_size: Option<u32>,
    /// Set the factor between adjacent scales of image pyramid.
    ///
    /// The value of the factor lies in (0.1, 0.99). For example, when it is set as 0.5,
    /// an input image of size w x h will be resized to 0.5w x 0.5h, 0.25w x 0.25h,  0.125w x 0.125h, etc.
    ///
    /// # Panics
    ///
    /// Panics if `scale_factor` is less than 0.01 or greater than 0.99
    pyramid_scale_factor: Option<f32>,
    /// Set the score threshold of detected faces.
    ///
    /// Detections with scores smaller than the threshold will not be returned.
    /// Typical threshold values include 0.95, 2.8, 4.5. One can adjust the
    /// threshold based on his or her own test set.
    ///
    /// Smaller values result in more detections (possibly increasing the number of false positives),
    /// larger values result in fewer detections (possibly increasing the number of false negatives).
    ///
    /// # Panics
    ///
    /// Panics if `thresh` is less than or equal to 0.
    score_thresh: Option<f64>,
}

impl TryFrom<FunctionArguments> for FaceDetectionArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialize arguments")
    }
}

impl FaceDetection {
    pub const NAME: &'static str = "builtin-face-detection";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(
        &self,
        arguments: FunctionArguments,
        _runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        let arguments = FaceDetectionArguments::try_from(arguments)?;
        let image = arguments.image;
        let img = image::load_from_memory(&image)?;

        let mut detector = rustface::create_default_detector()?;
        if let Some(window_size) = arguments.window_size {
            detector.set_window_size(window_size);
        }
        if let (Some(step_x), Some(step_y)) =
            (arguments.slide_window_step_x, arguments.slide_window_step_y)
        {
            detector.set_slide_window_step(step_x, step_y);
        }
        if let Some(min_face_size) = arguments.min_face_size {
            detector.set_min_face_size(min_face_size);
        }
        if let Some(max_face_size) = arguments.max_face_size {
            detector.set_max_face_size(max_face_size);
        }
        if let Some(pyramid_scale_factor) = arguments.pyramid_scale_factor {
            detector.set_pyramid_scale_factor(pyramid_scale_factor);
        }
        if let Some(score_thresh) = arguments.score_thresh {
            detector.set_score_thresh(score_thresh);
        }

        let faces = rustface::detect_faces(&mut *detector, img);
        let result = serde_json::to_string(&faces)?;

        Ok(result)
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use serde_json::json;
    use std::untrusted::fs;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_face_detection)
    }

    fn test_face_detection() {
        let input = "fixtures/functions/face_detection/input.jpg";
        let image = fs::read(input).unwrap();
        let arguments = FunctionArguments::from_json(json!({
            "image": &image,
            "min_face_size": 20,
            "score_thresh": 2.0,
            "pyramid_scale_factor": 0.8,
            "slide_window_step_x": 4,
            "slide_window_step_y": 4
        }))
        .unwrap();

        let input_files = StagedFiles::new(hashmap!());
        let output_files = StagedFiles::new(hashmap!());
        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));

        let result = FaceDetection::new().run(arguments, runtime).unwrap();
        let json_result: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json_result.as_array().unwrap().len(), 29);
    }
}
