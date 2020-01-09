use anyhow::{Error, Result};

pub mod proto {
    #![allow(clippy::all)]
    include!(concat!(
        env!("OUT_DIR"),
        "/teaclave_authorization_service_proto.rs"
    ));
}

pub use proto::TeaclaveAuthorization;
pub use proto::TeaclaveAuthorizationRequest;
pub use proto::TeaclaveAuthorizationResponse;

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct UserLoginRequest {
    pub id: std::string::String,
    pub password: std::string::String,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct UserLoginResponse {
    pub token: std::string::String,
}

impl std::convert::TryFrom<proto::UserLoginRequest> for UserLoginRequest {
    type Error = Error;

    fn try_from(proto: proto::UserLoginRequest) -> Result<Self> {
        let ret = Self {
            id: proto.id,
            password: proto.password,
        };

        Ok(ret)
    }
}

impl From<UserLoginRequest> for proto::UserLoginRequest {
    fn from(request: UserLoginRequest) -> Self {
        Self {
            id: request.id,
            password: request.password,
        }
    }
}

impl From<UserLoginResponse> for proto::UserLoginResponse {
    fn from(response: UserLoginResponse) -> Self {
        Self {
            token: response.token,
        }
    }
}
