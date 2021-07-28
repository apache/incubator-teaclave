#!/usr/bin/env python3
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

import numpy as np
import tvm
from PIL import Image
from tvm.contrib import graph_executor

lib_path = "./outlib/graph.o.so"
param_path = "./outlib/graph.params"
json_path = "./outlib/graph.json"
img_path = "./data/img_10.jpg"

loaded_lib = tvm.runtime.load_module(lib_path)
print(loaded_lib)

dev = tvm.runtime.cpu()
module = graph_executor.create(open(json_path).read(), loaded_lib, dev)

loaded_param = bytearray(open(param_path, "rb").read())
module.load_params(loaded_param)

# Resize it to 28X28
resized_image = Image.open(img_path).resize((28, 28))
img_data = np.asarray(resized_image).astype("float32") / 255
img_data = np.reshape(img_data, (1, 1, 28, 28))

print(loaded_lib)

module.set_input("Input3", img_data)
module.run()

output_shape = (1, 10)
tvm_output = module.get_output(0, tvm.nd.empty(output_shape)).numpy()

print(tvm_output)
