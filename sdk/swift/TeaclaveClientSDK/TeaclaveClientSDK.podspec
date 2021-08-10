# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

Pod::Spec.new do |s|
  s.name = "TeaclaveClientSDK"
  s.version = "0.2.0"
  s.summary = "Teaclave Client SDK."
  s.homepage = "https://teaclave.apache.org"
  s.license = "Apache-2.0"
  s.author = { "Teaclave Contributors" => "dev@teaclave.apache.org" }
  s.ios.deployment_target = '13.0'
  s.source = { :git => "https://github.com/apache/incubator-teaclave.git", :tag => "v0.2.0" }
  s.source_files  = "TeaclaveClietnSDK", "TeaclaveClientSDK/**/*.{h,swift}", "External"
  s.module_map = 'TeaclaveClientSDK/TeaclaveClientSDK.modulemap'
  s.vendored_libraries= 'External/libteaclave_client_sdk.a'
  s.requires_arc = true
  s.static_framework = true
  s.dependency 'OpenSSL-Universal', '~> 1.0.0'
  s.library = 'c++'
end
