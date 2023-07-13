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

macro_rules! include_proto {
    ($package: tt) => {
        include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
    };
}

macro_rules! impl_custom_client {
    ($target: ident) => {
        impl<T> $target<T>
        where
            T: tonic::client::GrpcService<tonic::body::BoxBody>,
            T::Error: Into<tonic::codegen::StdError>,
            T::ResponseBody: tonic::codegen::Body<Data = tonic::codegen::Bytes> + Send + 'static,
            <T::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + Send,
        {
            pub fn new_with_builtin_config(inner: T) -> Self {
                Self::new(inner)
                    .max_decoding_message_size(
                        teaclave_config::build::GRPC_CONFIG.max_decoding_message_size,
                    )
                    .max_encoding_message_size(
                        teaclave_config::build::GRPC_CONFIG.max_encoding_message_size,
                    )
            }
        }
    };
}

macro_rules! impl_custom_server {
    ($target: ident, $trait: ident) => {
        impl<T: $trait> $target<T> {
            pub fn new_with_builtin_config(inner: T) -> Self {
                Self::new(inner)
                    .max_decoding_message_size(
                        teaclave_config::build::GRPC_CONFIG.max_decoding_message_size,
                    )
                    .max_encoding_message_size(
                        teaclave_config::build::GRPC_CONFIG.max_encoding_message_size,
                    )
            }
        }
    };
}
