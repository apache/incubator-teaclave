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

use crate::user_info::UserInfo;
use rusty_leveldb::DB;
use std::prelude::v1::*;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DBError {
    #[error("user not exist")]
    UserNotExist,
    #[error("user exist")]
    UserExist,
    #[error("mpsc error")]
    MpscError,
    #[error("leveldb error")]
    LevelDbError,
    #[error("invalid response")]
    InvalidResponse,
    #[error("invalid request")]
    InvalidRequest,
}

#[derive(Clone)]
struct GetRequest {
    key: Vec<u8>,
}
#[derive(Clone)]
struct GetResponse {
    value: Vec<u8>,
}

#[derive(Clone)]
struct CreateRequest {
    key: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Clone)]
enum DBRequest {
    Get(GetRequest),
    Create(CreateRequest),
    Ping,
}

#[derive(Clone)]
enum DBResponse {
    Get(GetResponse),
    Create,
    Ping,
}

#[derive(Clone)]
struct DBCall {
    pub sender: Sender<Result<DBResponse, DBError>>,
    pub request: DBRequest,
}

pub struct Database {
    sender: Sender<DBCall>,
}

impl Database {
    pub fn open() -> Option<Self> {
        let (sender, receiver) = channel();
        thread::spawn(move || {
            let opt = rusty_leveldb::in_memory();
            let mut database = DB::open("authentication_db", opt).unwrap();
            loop {
                let call: DBCall = match receiver.recv() {
                    Ok(req) => req,
                    Err(e) => {
                        error!("mspc receive error: {}", e);
                        break;
                    }
                };
                let sender = call.sender;
                let response = match call.request {
                    DBRequest::Get(request) => match database.get(&request.key) {
                        Some(value) => Ok(DBResponse::Get(GetResponse { value })),
                        None => Err(DBError::UserNotExist),
                    },
                    DBRequest::Create(request) => match database.get(&request.key) {
                        Some(_) => Err(DBError::UserExist),
                        None => match database.put(&request.key, &request.value) {
                            Ok(_) => Ok(DBResponse::Create),
                            Err(_) => Err(DBError::LevelDbError),
                        },
                    },
                    DBRequest::Ping => Ok(DBResponse::Ping),
                };
                match sender.send(response) {
                    Ok(_) => (),
                    Err(e) => error!("mpsc send error: {}", e),
                }
            }
        });

        let database = Self { sender };
        let client = database.get_client();
        // check whether the db is opened successfuly
        match client.ping() {
            Ok(_) => Some(database),
            Err(_) => None,
        }
    }

    pub fn get_client(&self) -> DBClient {
        DBClient {
            sender: self.sender.clone(),
        }
    }
}

#[derive(Clone)]
pub struct DBClient {
    sender: Sender<DBCall>,
}

impl DBClient {
    pub fn get_user(&self, id: &str) -> Result<UserInfo, DBError> {
        let (sender, receiver) = channel();
        let request = DBRequest::Get(GetRequest {
            key: id.as_bytes().to_vec(),
        });
        let call = DBCall { sender, request };
        self.sender.send(call).map_err(|_| DBError::MpscError)?;
        let result = receiver.recv().map_err(|_| DBError::MpscError)?;
        let db_response = result?;
        match db_response {
            DBResponse::Get(response) => {
                let user = serde_json::from_slice(&response.value)
                    .map_err(|_| DBError::InvalidResponse)?;
                Ok(user)
            }
            _ => Err(DBError::UserNotExist),
        }
    }

    pub fn create_user(&self, user: &UserInfo) -> Result<(), DBError> {
        let (sender, receiver) = channel();
        let user_bytes = serde_json::to_vec(&user).map_err(|_| DBError::InvalidRequest)?;
        let request = DBRequest::Create(CreateRequest {
            key: user.id.as_bytes().to_vec(),
            value: user_bytes.to_vec(),
        });
        let call = DBCall { sender, request };
        self.sender.send(call).map_err(|_| DBError::MpscError)?;
        let result = receiver.recv().map_err(|_| DBError::MpscError)?;
        let db_response = result?;
        match db_response {
            DBResponse::Create => Ok(()),
            _ => Err(DBError::InvalidResponse),
        }
    }

    // use to check whether the db is opened successfully
    fn ping(&self) -> Result<(), DBError> {
        let (sender, receiver) = channel();
        let request = DBRequest::Ping;
        let call = DBCall { sender, request };
        self.sender.send(call).map_err(|_| DBError::MpscError)?;
        let result = receiver.recv().map_err(|_| DBError::MpscError)?;
        let db_response = result?;
        match db_response {
            DBResponse::Ping => Ok(()),
            _ => Err(DBError::InvalidResponse),
        }
    }
}
