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

public enum TeaclaveClientError: Error {
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

public struct RegisterFunctionRequest: Encodable {
    public let request: String = "register_function_request"
    public let name: String
    public let description: String
    public let executor_type: String
    public let `public`: Bool = true
    public let payload: [Int]
    public let arguments: Set<String>
    public let inputs: [[String: String]]
    public let outputs: [[String: String]]

    public init(name: String, description: String, executor_type: String, payload: [Int], arguments: Set<String>, inputs: [[String: String]], outputs: [[String: String]]) {
        self.name = name
        self.description = description
        self.executor_type = executor_type
        self.payload = payload
        self.arguments = arguments
        self.inputs = inputs
        self.outputs = outputs
    }
}

public struct RegisterFunctionResponse: Codable {
    public let function_id: String

    public init(function_id: String) {
        self.function_id = function_id
    }
}

public struct OwnerList: Encodable {
    public let data_name: String
    public let uids: [String]

    public init(data_name: String, uids: [String]) {
        self.data_name = data_name
        self.uids = uids
    }
}

public struct CreateTaskRequest: Encodable {
    public let request: String = "register_create_task"
    public let function_id: String
    public let function_arguments: String
    public let executor: String
    public let inputs_ownership: [OwnerList]
    public let outputs_ownership: [OwnerList]

    public init(function_id: String, function_arguments: String, executor: String, inputs_ownership: [OwnerList], outputs_ownership: [OwnerList]) {
        self.function_id = function_id
        self.function_arguments = function_arguments
        self.executor = executor
        self.inputs_ownership = inputs_ownership
        self.outputs_ownership = outputs_ownership
    }
}

public struct CreateTaskResponse: Codable {
    public let task_id: String

    public init(task_id: String) {
        self.task_id = task_id
    }
}

public struct CryptoInfo: Codable {
    public let schema: String
    public let key: [Int]
    public let iv: [Int]

    public init(schema: String, key: [Int], iv: [Int]) {
        self.schema = schema
        self.key = key
        self.iv = iv
    }
}

public struct RegisterInputFileRequest: Encodable {
    public let request: String = "register_input_file"
    public let url: String
    public let cmac: [Int]
    public let crypto_info: CryptoInfo

    public init(url: String, cmac: [Int], crypto_info: CryptoInfo) {
        self.url = url
        self.cmac = cmac
        self.crypto_info = crypto_info
    }
}

public struct RegisterInputFileResponse: Codable {
    public let data_id: String

    public init(data_id: String) {
        self.data_id = data_id
    }
}

public struct RegisterOutputFileRequest: Encodable {
    public let request: String = "register_output_file"
    public let url: String
    public let cmac: [Int]
    public let crypto_info: CryptoInfo

    public init(url: String, cmac: [Int], crypto_info: CryptoInfo) {
        self.url = url
        self.cmac = cmac
        self.crypto_info = crypto_info
    }
}

public struct RegisterOutputFileResponse: Codable {
    public let data_id: String

    public init(data_id: String) {
        self.data_id = data_id
    }
}

public struct DataMap: Codable {
    public let data_name: String
    public let data_id: String

    public init(data_name: String, data_id: String) {
        self.data_name = data_name
        self.data_id = data_id
    }
}

public struct AssignDataRequest: Encodable {
    public let request: String = "assign_data"
    public let task_id: String
    public let inputs: [DataMap]
    public let outputs: [DataMap]

    public init(task_id: String, inputs: [DataMap], outputs: [DataMap]) {
        self.task_id = task_id
        self.inputs = inputs
        self.outputs = outputs
    }
}

public struct ApproveTaskRequest: Encodable {
    public let request: String = "approve_task"
    public let task_id: String

    public init(task_id: String) {
        self.task_id = task_id
    }
}

public class AuthenticationClient {
    internal let cClient: OpaquePointer

    public init?(address: String, enclave_info_path: String, as_root_ca_cert_path: String) {
        let address_c_string = address.cString(using: .ascii)
        let enclave_info_path_address_c_string = enclave_info_path.cString(using: .ascii)
        let as_root_ca_cert_c_string = as_root_ca_cert_path.cString(using: .ascii)
        let cClient = teaclave_connect_authentication_service(address_c_string, enclave_info_path_address_c_string, as_root_ca_cert_c_string)
        guard let client = cClient else {
            return nil
        }
        self.cClient = client
    }

    public func register(id: String, password: String) -> Result<Void, TeaclaveClientError> {
        let id_c_string = id.cString(using: .ascii)
        let password_c_string = password.cString(using: .ascii)

        let ret = teaclave_user_register(cClient, id_c_string, password_c_string)
        guard ret == 0 else {
            return Result.failure(.userRegisterError)
        }
        return Result.success(())
    }

    public func login(id: String, password: String) -> Result<String, TeaclaveClientError> {
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
        let token = String(cString: token_c_string)
        token_c_string.deallocate()
        token_len.deallocate()

        return Result.success(token)
    }

    deinit {
        teaclave_close_authentication_service(cClient)
    }
}

public class FrontendClient {
    internal let cClient: OpaquePointer

    public init?(address: String, enclave_info_path: String, as_root_ca_cert_path: String) {
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

    public func set_credential(id: String, token: String) -> Result<Void, TeaclaveClientError> {
        let id_c_string = id.cString(using: .ascii)
        let token_c_string = token.cString(using: .ascii)

        let ret = teaclave_set_credential(cClient, id_c_string, token_c_string)
        guard ret == 0 else {
            return Result.failure(.setCredentialError)
        }
        return Result.success(())
    }

    public func register_function(with request: RegisterFunctionRequest) -> Result<RegisterFunctionResponse, TeaclaveClientError> {
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

    public func create_task(with request: CreateTaskRequest) -> Result<CreateTaskResponse, TeaclaveClientError> {
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

    public func register_input_file(with request: RegisterInputFileRequest) -> Result<RegisterInputFileResponse, TeaclaveClientError> {
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

    public func register_output_file(with request: RegisterOutputFileRequest) -> Result<RegisterOutputFileResponse, TeaclaveClientError> {
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

    public func assign_data(with request: AssignDataRequest) -> Result<Void, TeaclaveClientError> {
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

    public func invoke_task(task_id: String) -> Result<Void, TeaclaveClientError> {
        let task_id_c_string = task_id.cString(using: .ascii)
        let ret = teaclave_invoke_task(cClient, task_id_c_string)
        guard ret == 0 else {
            return Result.failure(.invokeTaskError)
        }

        return Result.success(())
    }

    public func approve_task(task_id: String) -> Result<Void, TeaclaveClientError> {
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

    public func get_task_result(task_id: String) -> Result<String, TeaclaveClientError> {
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
