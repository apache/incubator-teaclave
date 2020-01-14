use anyhow::{Error, Result};

pub mod proto {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/teaclave_common_proto.rs"));
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct UserCredential {
    pub id: std::string::String,
    pub token: std::string::String,
}

impl std::convert::TryFrom<proto::UserCredential> for UserCredential {
    type Error = Error;

    fn try_from(proto: proto::UserCredential) -> Result<Self> {
        let ret = Self {
            id: proto.id,
            token: proto.token,
        };

        Ok(ret)
    }
}

impl From<UserCredential> for proto::UserCredential {
    fn from(request: UserCredential) -> Self {
        Self {
            id: request.id,
            token: request.token,
        }
    }
}
