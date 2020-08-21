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

import os
import sys
import json

from PIL import Image, ImageDraw
import requests

from teaclave import (AuthenticationService, FrontendService, OwnerList,
                      AuthenticationClient, FrontendClient, DataMap,
                      FunctionInput)
from utils import (AUTHENTICATION_SERVICE_ADDRESS, FRONTEND_SERVICE_ADDRESS,
                   AS_ROOT_CA_CERT_PATH, ENCLAVE_INFO_PATH, USER_ID,
                   USER_PASSWORD)


class BuiltinFaceDetectionExample:
    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password

    def detect_face(self, image):
        client = AuthenticationService(
            AUTHENTICATION_SERVICE_ADDRESS, AS_ROOT_CA_CERT_PATH,
            ENCLAVE_INFO_PATH).connect().get_client()

        print("[+] registering user")
        client.user_register(self.user_id, self.user_password)

        print("[+] login")
        token = client.user_login(self.user_id, self.user_password)

        client = FrontendService(FRONTEND_SERVICE_ADDRESS,
                                 AS_ROOT_CA_CERT_PATH,
                                 ENCLAVE_INFO_PATH).connect().get_client()
        metadata = {"id": self.user_id, "token": token}
        client.metadata = metadata

        print("[+] registering function")
        function_id = client.register_function(
            name="builtin-face-detection",
            description="Native Face Detection Function",
            executor_type="builtin",
            inputs=[],
            arguments=[
                "image", "min_face_size", "score_thresh",
                "pyramid_scale_factor", "slide_window_step_x",
                "slide_window_step_y"
            ])

        print("[+] creating task")
        task_id = client.create_task(function_id=function_id,
                                     function_arguments={
                                         "image": image,
                                         "min_face_size": 20,
                                         "score_thresh": 2.0,
                                         "pyramid_scale_factor": 0.8,
                                         "slide_window_step_x": 4,
                                         "slide_window_step_y": 4
                                     },
                                     inputs_ownership=[],
                                     executor="builtin")

        print("[+] invoking task")
        client.invoke_task(task_id)

        print("[+] getting result")
        result = client.get_task_result(task_id)
        print("[+] done")

        return bytes(result)


def main():
    img_file_name = 'in.jpg'

    if not os.path.isfile(img_file_name):
        image_url = 'https://upload.wikimedia.org/wikipedia/commons/thumb/6/6e/Solvay_conference_1927.jpg/1400px-Solvay_conference_1927.jpg'
        print("[+] retrieving image from url:", image_url)

        response = requests.get(image_url).content
        with open(img_file_name, 'wb') as file:
            file.write(response)
        print("[+] image saved to", img_file_name)
    else:
        print("[+] using cached file", img_file_name)

    with open(img_file_name, 'rb') as fin:
        image_data = fin.read()
        example = BuiltinFaceDetectionExample(USER_ID, USER_PASSWORD)

        rt = example.detect_face(list(image_data))

        print("[+] function return:", rt)

        bboxes = json.loads(rt)

        img = Image.open(img_file_name).convert('RGB')
        draw = ImageDraw.Draw(img)

        for bbox in bboxes:
            box = bbox['bbox']
            draw.rectangle([
                box['x'], box['y'], box['x'] + box['height'],
                box['y'] + box['width']
            ],
                           outline='red',
                           width=2)

        img.save('out.jpg', 'JPEG')
        print("[+] detection result saved to out.jpg")


if __name__ == '__main__':
    main()
