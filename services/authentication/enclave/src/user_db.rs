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
use rusty_leveldb;
use std::prelude::v1::*;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum DbError {
    #[error("user not exist")]
    UserNotExist,
    #[error("user exist")]
    UserExist,
    #[error("mpsc error")]
    ConnectionError,
    #[error("leveldb error")]
    LevelDbInternalError,
    #[error("invalid response")]
    InvalidResponse,
    #[error("invalid request")]
    InvalidRequest,
}

impl<T> From<std::sync::mpsc::SendError<T>> for DbError {
    fn from(_error: std::sync::mpsc::SendError<T>) -> Self {
        DbError::ConnectionError
    }
}

impl From<std::sync::mpsc::RecvError> for DbError {
    fn from(_error: std::sync::mpsc::RecvError) -> Self {
        DbError::ConnectionError
    }
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
enum DbRequest {
    Get(GetRequest),
    Create(CreateRequest),
    Ping,
}

#[derive(Clone)]
enum DbResponse {
    Get(GetResponse),
    Create,
    Ping,
}

#[derive(Clone)]
struct DBCall {
    pub sender: Sender<Result<DbResponse, DbError>>,
    pub request: DbRequest,
}

pub(crate) struct Database {
    sender: Sender<DBCall>,
}

impl Database {
    pub(crate) fn open() -> Result<Self, DbError> {
        let (sender, receiver) = channel();
        thread::spawn(move || {
            let opt = rusty_leveldb::in_memory();
            let mut database = rusty_leveldb::DB::open("authentication_db", opt).unwrap();
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
                    DbRequest::Get(request) => match database.get(&request.key) {
                        Some(value) => Ok(DbResponse::Get(GetResponse { value })),
                        None => Err(DbError::UserNotExist),
                    },
                    DbRequest::Create(request) => match database.get(&request.key) {
                        Some(_) => Err(DbError::UserExist),
                        None => match database.put(&request.key, &request.value) {
                            Ok(_) => Ok(DbResponse::Create),
                            Err(_) => Err(DbError::LevelDbInternalError),
                        },
                    },
                    DbRequest::Ping => Ok(DbResponse::Ping),
                };
                match sender.send(response) {
                    Ok(_) => (),
                    Err(e) => error!("mpsc send error: {}", e),
                }
            }
        });

        let database = Self { sender };
        let client = database.get_client();

        // Check whether the user database is successfully opened.
        client.ping()?;

        Ok(database)
    }

    pub(crate) fn get_client(&self) -> DbClient {
        DbClient {
            sender: self.sender.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct DbClient {
    sender: Sender<DBCall>,
}

impl DbClient {
    pub(crate) fn get_user(&self, id: &str) -> Result<UserInfo, DbError> {
        let (sender, receiver) = channel();
        let request = DbRequest::Get(GetRequest {
            key: id.as_bytes().to_vec(),
        });
        let call = DBCall { sender, request };
        self.sender.send(call)?;
        let result = receiver.recv()?;
        let db_response = result?;
        match db_response {
            DbResponse::Get(response) => {
                let user = serde_json::from_slice(&response.value)
                    .map_err(|_| DbError::InvalidResponse)?;
                Ok(user)
            }
            _ => Err(DbError::UserNotExist),
        }
    }

    pub(crate) fn create_user(&self, user: &UserInfo) -> Result<(), DbError> {
        let (sender, receiver) = channel();
        let user_bytes = serde_json::to_vec(&user).map_err(|_| DbError::InvalidRequest)?;
        let request = DbRequest::Create(CreateRequest {
            key: user.id.as_bytes().to_vec(),
            value: user_bytes.to_vec(),
        });
        let call = DBCall { sender, request };
        self.sender.send(call)?;
        let result = receiver.recv()?;
        let db_response = result?;
        match db_response {
            DbResponse::Create => Ok(()),
            _ => Err(DbError::InvalidResponse),
        }
    }

    // Check whether the database is opened successfully.
    fn ping(&self) -> Result<(), DbError> {
        let (sender, receiver) = channel();
        let request = DbRequest::Ping;
        let call = DBCall { sender, request };
        self.sender.send(call)?;
        let result = receiver.recv()?;
        let db_response = result?;
        match db_response {
            DbResponse::Ping => Ok(()),
            _ => Err(DbError::InvalidResponse),
        }
    }
}
