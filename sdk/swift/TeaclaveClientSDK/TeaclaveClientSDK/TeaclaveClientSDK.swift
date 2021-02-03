//
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
//

import Foundation
import TeaclaveClientSDK.CTeaclaveClientSDK

enum TeaclaveClientError: Error {
    case userLoginError
    case userRegisterError
    case setCredentialError
    case registerFunctionError
    case createTaskError
    case invokeTaskError
    case getTaskResultError
    case registerInputFileError
    case registerOutputFileError
    case assignDataError
    case approveTaskError
}

struct RegisterFunctionRequest: Encodable {
    let request: String = "register_function_request"
    let name: String
    let description: String
    let executor_type: String
    let `public`: Bool = true
    let payload: Array<Int>
    let arguments: Set<String>
    let inputs: Array<Dictionary<String, String>>
    let outputs: Array<Dictionary<String, String>>
}

struct RegisterFunctionResponse: Codable {
    let function_id: String
}

struct OwnerList: Encodable {
    let data_name: String
    let uids: Array<String>
}

struct CreateTaskRequest: Encodable {
    let request: String = "register_create_task"
    let function_id: String
    let function_arguments: String
    let executor: String
    let inputs_ownership: Array<OwnerList>
    let outputs_ownership: Array<OwnerList>
}

struct CreateTaskResponse: Codable {
    let task_id: String
}

struct CryptoInfo: Codable {
    let schema: String
    let key: Array<Int>
    let iv: Array<Int>
}

struct RegisterInputFileRequest: Encodable {
    let request: String = "register_input_file"
    let url: String
    let cmac: String
    let crypto_info: CryptoInfo
}

struct RegisterInputFileResponse: Codable {
    let data_id: String
}

struct RegisterOutputFileRequest: Encodable {
    let request: String = "register_output_file"
    let url: String
    let cmac: String
    let crypto_info: CryptoInfo
}

struct RegisterOutputFileResponse: Codable {
    let data_id: String
}

struct DataMap: Codable {
    let data_name: String
    let data_id: String
}

struct AssignDataRequest: Encodable {
    let request: String = "assign_data"
    let task_id: String
    let inputs: Array<DataMap>
    let outputs: Array<DataMap>
}

struct ApproveTaskRequest: Encodable {
    let request: String = "approve_task"
    let task_id: String
}

class AuthenticationClient {
    internal let cClient: OpaquePointer

    init?(address: String, enclave_info_path: String, as_root_ca_cert_path: String) {
        let address_c_string = address.cString(using: .ascii)
        let enclave_info_path_address_c_string = enclave_info_path.cString(using: .ascii)
        let as_root_ca_cert_c_string = as_root_ca_cert_path.cString(using: .ascii)
        let cClient = teaclave_connect_authentication_service(address_c_string, enclave_info_path_address_c_string, as_root_ca_cert_c_string)
        guard let client = cClient else {
            return nil
        }
        self.cClient = client
    }

    func register(id: String, password: String) -> Result<(), TeaclaveClientError> {
        let id_c_string = id.cString(using: .ascii)
        let password_c_string = password.cString(using: .ascii)

        let ret = teaclave_user_register(cClient, id_c_string, password_c_string)
        guard ret == 0 else {
            return Result.failure(.userRegisterError)
        }
        return Result.success(())
    }

    func login(id: String, password: String) -> Result<String, TeaclaveClientError> {
        let id_c_string = id.cString(using: .ascii)
        let password_c_string = password.cString(using: .ascii)
        let token_c_string = UnsafeMutablePointer<CChar>.allocate(capacity: 1024)
        let token_len = UnsafeMutablePointer<Int>.allocate(capacity: 1)
        token_len.pointee = 1024
        let ret = teaclave_user_login(cClient, id_c_string, password_c_string, token_c_string, token_len)
        guard ret == 0 else {
            token_c_string.deallocate()
            token_len.deallocate()
            return Result.failure(.userLoginError)
        }
        let token =  String(cString: token_c_string)
        token_c_string.deallocate()
        token_len.deallocate()

        return Result.success(token)
    }

    deinit {
        teaclave_close_authentication_service(cClient)
    }
}

class FrontendClient {
    internal let cClient: OpaquePointer

    init?(address: String, enclave_info_path: String, as_root_ca_cert_path: String) {
        let address_c_string = address.cString(using: .ascii)
        let enclave_info_path_address_c_string = enclave_info_path.cString(using: .ascii)
        let as_root_ca_cert_c_string = as_root_ca_cert_path.cString(using: .ascii)
        let cClient = teaclave_connect_frontend_service(address_c_string, enclave_info_path_address_c_string, as_root_ca_cert_c_string)
        guard let client = cClient else {
            return nil
        }
        self.cClient = client
    }

    deinit {
        teaclave_close_frontend_service(cClient)
    }

    func set_credential(id: String, token: String) -> Result<(), TeaclaveClientError> {
        let id_c_string = id.cString(using: .ascii)
        let token_c_string = token.cString(using: .ascii)

        let ret = teaclave_set_credential(cClient, id_c_string, token_c_string)
        guard ret == 0 else {
            return Result.failure(.setCredentialError)
        }
        return Result.success(())
    }

    func register_function(with request: RegisterFunctionRequest) -> Result<RegisterFunctionResponse, TeaclaveClientError> {
        let data = try! JSONEncoder().encode(request)
        let json_string = String(data: data, encoding: .ascii)!

        let request_c_string = json_string.cString(using: .ascii)
        let response_c_string = UnsafeMutablePointer<CChar>.allocate(capacity: 1024)
        let response_len = UnsafeMutablePointer<Int>.allocate(capacity: 1)
        response_len.pointee = 1024
        let ret = teaclave_register_function_serialized(cClient, request_c_string, response_c_string, response_len)
        guard ret == 0 else {
            return Result.failure(.registerFunctionError)
        }

        let response_string = String(cString: response_c_string)
        let response = try! JSONDecoder().decode(RegisterFunctionResponse.self, from: response_string.data(using: .ascii)!)

        return Result.success(response)
    }

    func create_task(with request: CreateTaskRequest) -> Result<CreateTaskResponse, TeaclaveClientError> {
        let data = try! JSONEncoder().encode(request)
        let json_string = String(data: data, encoding: .ascii)!

        let request_c_string = json_string.cString(using: .ascii)
        let response_c_string = UnsafeMutablePointer<CChar>.allocate(capacity: 10240)
        let response_len = UnsafeMutablePointer<Int>.allocate(capacity: 1)
        response_len.pointee = 10240
        let ret = teaclave_create_task_serialized(cClient, request_c_string, response_c_string, response_len)
        guard ret == 0 else {
            return Result.failure(.createTaskError)
        }

        let response_string = String(cString: response_c_string)

        let response = try! JSONDecoder().decode(CreateTaskResponse.self, from: response_string.data(using: .ascii)!)

        return Result.success(response)
    }

    func register_input_file(with request: RegisterInputFileRequest) -> Result<RegisterInputFileResponse, TeaclaveClientError> {
        let data = try! JSONEncoder().encode(request)
        let json_string = String(data: data, encoding: .ascii)!

        let request_c_string = json_string.cString(using: .ascii)
        let response_c_string = UnsafeMutablePointer<CChar>.allocate(capacity: 10240)
        let response_len = UnsafeMutablePointer<Int>.allocate(capacity: 1)
        response_len.pointee = 10240
        let ret = teaclave_register_input_file_serialized(cClient, request_c_string, response_c_string, response_len)
        guard ret == 0 else {
            return Result.failure(.registerInputFileError)
        }

        let response_string = String(cString: response_c_string)

        let response = try! JSONDecoder().decode(RegisterInputFileResponse.self, from: response_string.data(using: .ascii)!)

        return Result.success(response)
    }

    func register_output_file(with request: RegisterOutputFileRequest) -> Result<RegisterOutputFileResponse, TeaclaveClientError> {
        let data = try! JSONEncoder().encode(request)
        let json_string = String(data: data, encoding: .ascii)!

        let request_c_string = json_string.cString(using: .ascii)
        let response_c_string = UnsafeMutablePointer<CChar>.allocate(capacity: 10240)
        let response_len = UnsafeMutablePointer<Int>.allocate(capacity: 1)
        response_len.pointee = 10240
        let ret = teaclave_register_output_file_serialized(cClient, request_c_string, response_c_string, response_len)
        guard ret == 0 else {
            return Result.failure(.registerOutputFileError)
        }

        let response_string = String(cString: response_c_string)

        let response = try! JSONDecoder().decode(RegisterOutputFileResponse.self, from: response_string.data(using: .ascii)!)

        return Result.success(response)
    }

    func assign_data(with request: AssignDataRequest) -> Result<(), TeaclaveClientError> {
        let data = try! JSONEncoder().encode(request)
        let json_string = String(data: data, encoding: .ascii)!

        let request_c_string = json_string.cString(using: .ascii)
        let response_c_string = UnsafeMutablePointer<CChar>.allocate(capacity: 10240)
        let response_len = UnsafeMutablePointer<Int>.allocate(capacity: 1)
        response_len.pointee = 10240
        let ret = teaclave_assign_data_serialized(cClient, request_c_string, response_c_string, response_len)
        guard ret == 0 else {
            return Result.failure(.assignDataError)
        }

        return Result.success(())
    }

    func invoke_task(task_id: String) -> Result<(), TeaclaveClientError> {
        let task_id_c_string = task_id.cString(using: .ascii)
        let ret = teaclave_invoke_task(cClient, task_id_c_string)
        guard ret == 0 else {
            return Result.failure(.invokeTaskError)
        }

        return Result.success(())
    }

    func approve_task(task_id: String) -> Result<(), TeaclaveClientError> {
        let request = ApproveTaskRequest(task_id: task_id)
        let data = try! JSONEncoder().encode(request)
        let json_string = String(data: data, encoding: .ascii)!

        let request_c_string = json_string.cString(using: .ascii)
        let response_c_string = UnsafeMutablePointer<CChar>.allocate(capacity: 10240)
        let response_len = UnsafeMutablePointer<Int>.allocate(capacity: 1)
        response_len.pointee = 10240
        let ret = teaclave_approve_task_serialized(cClient, request_c_string, response_c_string, response_len)
        guard ret == 0 else {
            return Result.failure(.approveTaskError)
        }
        return Result.success(())
    }

    func get_task_result(task_id: String) -> Result<String, TeaclaveClientError> {
        let task_id_c_string = task_id.cString(using: .ascii)
        let response_c_string = UnsafeMutablePointer<CChar>.allocate(capacity: 10240)
        let response_len = UnsafeMutablePointer<Int>.allocate(capacity: 1)
        response_len.pointee = 10240
        let ret = teaclave_get_task_result(cClient, task_id_c_string, response_c_string, response_len)
        guard ret == 0 else {
            return Result.failure(.getTaskResultError)
        }

        return Result.success(String(cString: response_c_string))
    }
}
