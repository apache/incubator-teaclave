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

import _numpypy as np
import marshal


def read_file_train(file_id):
    with teaclave_open(file_id, "rb") as rdr:
        featureData = []
        labelData = []
        while True:
            line = rdr.readline()
            if not line:
                break
            else:
                line = line.strip().split(',')
                featureData.append(line[:-1])
                labelData.append(line[-1])
        label = np.multiarray.array(labelData, dtype='float64').reshape(-1, 1)
        feature = np.multiarray.array(featureData, dtype='float64')

    return feature, label


def read_file_predict(file_id, params_id, scaler_id):
    params = None
    scaler = None
    with teaclave_open(params_id, "rb") as rdr0:
        params = rdr0.read()
    with teaclave_open(scaler_id, "rb") as rdr1:
        scaler = rdr1.read()

    params = marshal.loads(params)
    scaler = marshal.loads(scaler)
    featureData = []
    with teaclave_open(file_id, "rb") as rdr2:
        while True:
            line = rdr2.readline()
            if not line:
                break
            else:
                featureData.append(line.strip().split(','))
    feature = np.multiarray.array(featureData, dtype='float64')
    return feature, params, scaler


def save_model(params, scaler, params_saved, scaler_saved):
    params = marshal.dumps(params)
    scaler = marshal.dumps(scaler)
    with teaclave_open(params_saved, "wb") as wtr:
        wtr.write(params)
    with teaclave_open(scaler_saved, "wb") as wtr:
        wtr.write(scaler)
    return


def minmaxscaler_train(input_array):
    array_max = input_array.max(0)
    array_min = input_array.min(0)
    scaler = {"max": array_max, "min": array_min}
    input_array = (input_array - array_min) / (array_max - array_min)
    return input_array, scaler


def minmaxscaler_predict(input_array, scaler):
    array_max = np.multiarray.frombuffer(scaler["max"])
    array_min = np.multiarray.frombuffer(scaler["min"])
    input_array = (input_array - array_min) / (array_max - array_min)
    return input_array


def sigmoid(z):
    a = 1 / (1 + np.umath.exp(-z))
    return a


def initialize_with_zeros(dim):
    w = np.multiarray.zeros((dim, 1))
    b = 0
    return w, b


def propagate(w, b, X, Y):
    m = X.shape[1]
    A = sigmoid(w.T.dot(X) + b)
    cost = -((Y * np.umath.log(A) + (1 - Y) * np.umath.log(1 - A)).sum()) / m

    dZ = A - Y
    dw = (X.dot(dZ.T)) / m
    db = (dZ.sum()) / m

    grads = {"dw": dw, "db": db}
    return grads, cost


def optimize(w, b, X, Y, num_iterations, learning_rate):
    costs = []
    for i in range(num_iterations):
        grads, cost = propagate(w, b, X, Y)
        dw = grads["dw"]
        db = grads["db"]
        w = w - learning_rate * dw
        b = b - learning_rate * db
    params = {"w": w, "b": b}
    return params


def logistic_model(feature, label, learning_rate=0.1, num_iterations=2000):
    dim = feature.shape[0]
    w, b = initialize_with_zeros(dim)
    params = optimize(w, b, feature, label, num_iterations, learning_rate)
    return params


def logistic_predict(feature, params):
    w = np.multiarray.frombuffer(params['w'])
    b = np.multiarray.frombuffer(params['b'])
    m = feature.shape[1]
    prediction = np.multiarray.zeros((1, m))

    A = sigmoid(w.T.dot(feature) + b)
    for i in range(m):
        if A[i] > 0.5:
            prediction[0, i] = 1
        else:
            prediction[0, i] = 0

    return prediction


def train(train_file, params_saved, scaler_saved):
    feature, label = read_file_train(train_file)
    feature, scaler = minmaxscaler_train(feature)
    feature = feature.T
    label = label.T
    params = logistic_model(feature,
                            label,
                            num_iterations=2000,
                            learning_rate=0.05)
    save_model(params, scaler, params_saved, scaler_saved)
    return


def predict(file_id, params_id, scaler_id):
    feature, params, scaler = read_file_predict(file_id, params_id, scaler_id)
    feature = minmaxscaler_predict(feature, scaler)
    feature = feature.T
    prediction = logistic_predict(feature, params)
    return prediction


def entrypoint(argv):

    assert len(argv) == 8
    for i in range(0, 4):
        if argv[2 * i] == "train_file":
            train_file = argv[2 * i + 1]
        elif argv[2 * i] == "predict_file":
            predict_file = argv[2 * i + 1]
        elif argv[2 * i] == "params_saved":
            params_saved = argv[2 * i + 1]
        elif argv[2 * i] == "scaler_saved":
            scaler_saved = argv[2 * i + 1]
        elif argv[2 * i] == "operation":
            reg_type = argv[2 * i + 1]
    if reg_type == "train":
        train(train_file, params_saved, scaler_saved)
        return "Training is finished!"
    elif reg_type == "predict":
        prediction = predict(predict_file, params_saved, scaler_saved)
        return str(prediction)
    else:
        return "NOT supported argv"
