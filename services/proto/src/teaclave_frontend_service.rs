use crate::teaclave_frontend_service_proto as proto;
use anyhow::anyhow;
use anyhow::{Error, Result};
use core::convert::TryInto;
use serde::{Deserialize, Serialize};
use teaclave_types::TeaclaveFileCryptoInfo;
use url::Url;

pub use proto::TeaclaveFrontend;
pub use proto::TeaclaveFrontendClient;
pub use proto::TeaclaveFrontendRequest;
pub use proto::TeaclaveFrontendResponse;

#[derive(Debug)]
pub struct RegisterInputFileRequest {
    pub url: Url,
    pub hash: std::string::String,
    pub crypto_info: TeaclaveFileCryptoInfo,
}

#[derive(Debug)]
pub struct RegisterInputFileResponse {
    pub data_id: std::string::String,
}

#[derive(Debug)]
pub struct RegisterOutputFileRequest {
    pub url: Url,
    pub crypto_info: TeaclaveFileCryptoInfo,
}

#[derive(Debug)]
pub struct RegisterOutputFileResponse {
    pub data_id: std::string::String,
}

#[derive(Debug)]
pub struct GetOutputFileRequest {
    pub data_id: std::string::String,
}

#[derive(Debug)]
pub struct GetOutputFileResponse {
    pub hash: std::string::String,
}

#[derive(Debug)]
pub struct GetFusionDataRequest {
    pub data_id: std::string::String,
}

#[derive(Debug)]
pub struct GetFusionDataResponse {
    pub hash: std::string::String,
    pub data_owner_id_list: std::vec::Vec<std::string::String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionInput {
    pub name: std::string::String,
    pub description: std::string::String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionOutput {
    pub name: std::string::String,
    pub description: std::string::String,
}

#[derive(Debug)]
pub struct RegisterFunctionRequest {
    pub name: std::string::String,
    pub description: std::string::String,
    pub payload: std::vec::Vec<u8>,
    pub is_public: bool,
    pub arg_list: std::vec::Vec<std::string::String>,
    pub input_list: std::vec::Vec<FunctionInput>,
    pub output_list: std::vec::Vec<FunctionOutput>,
}

#[derive(Debug)]
pub struct RegisterFunctionResponse {
    pub function_id: std::string::String,
}

#[derive(Debug)]
pub struct GetFunctionRequest {
    pub function_id: std::string::String,
}

#[derive(Debug)]
pub struct GetFunctionResponse {
    pub name: std::string::String,
    pub description: std::string::String,
    pub owner: std::string::String,
    pub payload: std::vec::Vec<u8>,
    pub is_public: bool,
    pub arg_list: std::vec::Vec<std::string::String>,
    pub input_list: std::vec::Vec<FunctionInput>,
    pub output_list: std::vec::Vec<FunctionOutput>,
}

impl std::convert::TryFrom<proto::RegisterInputFileRequest> for RegisterInputFileRequest {
    type Error = Error;

    fn try_from(proto: proto::RegisterInputFileRequest) -> Result<Self> {
        let ret = Self {
            url: Url::parse(&proto.url)?,
            hash: proto.hash,
            crypto_info: proto
                .crypto_info
                .ok_or_else(|| anyhow!("missing crypto_info"))?
                .try_into()?,
        };

        Ok(ret)
    }
}

impl From<RegisterInputFileRequest> for proto::RegisterInputFileRequest {
    fn from(request: RegisterInputFileRequest) -> Self {
        Self {
            url: request.url.into_string(),
            hash: request.hash,
            crypto_info: Some(request.crypto_info.into()),
        }
    }
}

impl std::convert::TryFrom<proto::RegisterInputFileResponse> for RegisterInputFileResponse {
    type Error = Error;

    fn try_from(proto: proto::RegisterInputFileResponse) -> Result<Self> {
        Ok(Self {
            data_id: proto.data_id,
        })
    }
}

impl From<RegisterInputFileResponse> for proto::RegisterInputFileResponse {
    fn from(request: RegisterInputFileResponse) -> Self {
        Self {
            data_id: request.data_id,
        }
    }
}

impl std::convert::TryFrom<proto::RegisterOutputFileRequest> for RegisterOutputFileRequest {
    type Error = Error;

    fn try_from(proto: proto::RegisterOutputFileRequest) -> Result<Self> {
        let ret = Self {
            url: Url::parse(&proto.url)?,
            crypto_info: proto
                .crypto_info
                .ok_or_else(|| anyhow!("missing crypto_info"))?
                .try_into()?,
        };

        Ok(ret)
    }
}

impl From<RegisterOutputFileRequest> for proto::RegisterOutputFileRequest {
    fn from(request: RegisterOutputFileRequest) -> Self {
        Self {
            url: request.url.into_string(),
            crypto_info: Some(request.crypto_info.into()),
        }
    }
}

impl std::convert::TryFrom<proto::RegisterOutputFileResponse> for RegisterOutputFileResponse {
    type Error = Error;

    fn try_from(proto: proto::RegisterOutputFileResponse) -> Result<Self> {
        Ok(Self {
            data_id: proto.data_id,
        })
    }
}

impl From<RegisterOutputFileResponse> for proto::RegisterOutputFileResponse {
    fn from(request: RegisterOutputFileResponse) -> Self {
        Self {
            data_id: request.data_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetOutputFileRequest> for GetOutputFileRequest {
    type Error = Error;

    fn try_from(proto: proto::GetOutputFileRequest) -> Result<Self> {
        let ret = Self {
            data_id: proto.data_id,
        };

        Ok(ret)
    }
}

impl From<GetOutputFileRequest> for proto::GetOutputFileRequest {
    fn from(request: GetOutputFileRequest) -> Self {
        Self {
            data_id: request.data_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetOutputFileResponse> for GetOutputFileResponse {
    type Error = Error;

    fn try_from(proto: proto::GetOutputFileResponse) -> Result<Self> {
        Ok(Self { hash: proto.hash })
    }
}

impl From<GetOutputFileResponse> for proto::GetOutputFileResponse {
    fn from(response: GetOutputFileResponse) -> Self {
        Self {
            hash: response.hash,
        }
    }
}

impl std::convert::TryFrom<proto::GetFusionDataRequest> for GetFusionDataRequest {
    type Error = Error;

    fn try_from(proto: proto::GetFusionDataRequest) -> Result<Self> {
        let ret = Self {
            data_id: proto.data_id,
        };

        Ok(ret)
    }
}

impl From<GetFusionDataRequest> for proto::GetFusionDataRequest {
    fn from(request: GetFusionDataRequest) -> Self {
        Self {
            data_id: request.data_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetFusionDataResponse> for GetFusionDataResponse {
    type Error = Error;

    fn try_from(proto: proto::GetFusionDataResponse) -> Result<Self> {
        Ok(Self {
            hash: proto.hash,
            data_owner_id_list: proto.data_owner_id_list,
        })
    }
}

impl From<GetFusionDataResponse> for proto::GetFusionDataResponse {
    fn from(response: GetFusionDataResponse) -> Self {
        Self {
            hash: response.hash,
            data_owner_id_list: response.data_owner_id_list,
        }
    }
}

impl std::convert::TryFrom<proto::FunctionInput> for FunctionInput {
    type Error = Error;

    fn try_from(proto: proto::FunctionInput) -> Result<Self> {
        let ret = Self {
            name: proto.name,
            description: proto.description,
        };

        Ok(ret)
    }
}

impl From<FunctionInput> for proto::FunctionInput {
    fn from(input: FunctionInput) -> Self {
        Self {
            name: input.name,
            description: input.description,
        }
    }
}

impl std::convert::TryFrom<proto::FunctionOutput> for FunctionOutput {
    type Error = Error;

    fn try_from(proto: proto::FunctionOutput) -> Result<Self> {
        let ret = Self {
            name: proto.name,
            description: proto.description,
        };

        Ok(ret)
    }
}

impl From<FunctionOutput> for proto::FunctionOutput {
    fn from(output: FunctionOutput) -> Self {
        Self {
            name: output.name,
            description: output.description,
        }
    }
}

impl std::convert::TryFrom<proto::RegisterFunctionRequest> for RegisterFunctionRequest {
    type Error = Error;

    fn try_from(proto: proto::RegisterFunctionRequest) -> Result<Self> {
        let input_list: Result<std::vec::Vec<FunctionInput>> = proto
            .input_list
            .into_iter()
            .map(FunctionInput::try_from)
            .collect();
        let output_list: Result<std::vec::Vec<FunctionOutput>> = proto
            .output_list
            .into_iter()
            .map(FunctionOutput::try_from)
            .collect();

        let ret = Self {
            name: proto.name,
            description: proto.description,
            payload: proto.payload,
            is_public: proto.is_public,
            arg_list: proto.arg_list,
            input_list: input_list?,
            output_list: output_list?,
        };
        Ok(ret)
    }
}

impl From<RegisterFunctionRequest> for proto::RegisterFunctionRequest {
    fn from(request: RegisterFunctionRequest) -> Self {
        let input_list: std::vec::Vec<proto::FunctionInput> = request
            .input_list
            .into_iter()
            .map(proto::FunctionInput::from)
            .collect();
        let output_list: std::vec::Vec<proto::FunctionOutput> = request
            .output_list
            .into_iter()
            .map(proto::FunctionOutput::from)
            .collect();

        Self {
            name: request.name,
            description: request.description,
            payload: request.payload,
            is_public: request.is_public,
            arg_list: request.arg_list,
            input_list,
            output_list,
        }
    }
}

impl std::convert::TryFrom<proto::RegisterFunctionResponse> for RegisterFunctionResponse {
    type Error = Error;

    fn try_from(proto: proto::RegisterFunctionResponse) -> Result<Self> {
        let ret = Self {
            function_id: proto.function_id,
        };

        Ok(ret)
    }
}

impl From<RegisterFunctionResponse> for proto::RegisterFunctionResponse {
    fn from(response: RegisterFunctionResponse) -> Self {
        Self {
            function_id: response.function_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetFunctionRequest> for GetFunctionRequest {
    type Error = Error;

    fn try_from(proto: proto::GetFunctionRequest) -> Result<Self> {
        let ret = Self {
            function_id: proto.function_id,
        };

        Ok(ret)
    }
}

impl From<GetFunctionRequest> for proto::GetFunctionRequest {
    fn from(request: GetFunctionRequest) -> Self {
        Self {
            function_id: request.function_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetFunctionResponse> for GetFunctionResponse {
    type Error = Error;

    fn try_from(proto: proto::GetFunctionResponse) -> Result<Self> {
        let input_list: Result<std::vec::Vec<FunctionInput>> = proto
            .input_list
            .into_iter()
            .map(FunctionInput::try_from)
            .collect();
        let output_list: Result<std::vec::Vec<FunctionOutput>> = proto
            .output_list
            .into_iter()
            .map(FunctionOutput::try_from)
            .collect();

        let ret = Self {
            name: proto.name,
            description: proto.description,
            owner: proto.owner,
            payload: proto.payload,
            is_public: proto.is_public,
            arg_list: proto.arg_list,
            input_list: input_list?,
            output_list: output_list?,
        };

        Ok(ret)
    }
}

impl From<GetFunctionResponse> for proto::GetFunctionResponse {
    fn from(response: GetFunctionResponse) -> Self {
        let input_list: std::vec::Vec<proto::FunctionInput> = response
            .input_list
            .into_iter()
            .map(proto::FunctionInput::from)
            .collect();
        let output_list: std::vec::Vec<proto::FunctionOutput> = response
            .output_list
            .into_iter()
            .map(proto::FunctionOutput::from)
            .collect();

        Self {
            name: response.name,
            description: response.description,
            owner: response.owner,
            payload: response.payload,
            is_public: response.is_public,
            arg_list: response.arg_list,
            input_list,
            output_list,
        }
    }
}
