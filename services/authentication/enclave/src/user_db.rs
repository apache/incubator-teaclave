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
use rusty_leveldb::LdbIterator;
use rusty_leveldb::DB;
use std::path::Path;
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
struct ListRequest {
    key: String,
}
#[derive(Clone)]
struct ListResponse {
    values: Vec<String>,
}

#[derive(Clone)]
struct CreateRequest {
    key: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Clone)]
struct UpdateRequest {
    key: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Clone)]
struct DeleteRequest {
    key: Vec<u8>,
}

#[derive(Clone)]
enum DbRequest {
    Get(GetRequest),
    Create(CreateRequest),
    Update(UpdateRequest),
    Delete(DeleteRequest),
    List(ListRequest),
    Ping,
}

#[derive(Clone)]
enum DbResponse {
    Get(GetResponse),
    List(ListResponse),
    Create,
    Delete,
    Update,
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

#[cfg(not(test_mode))]
pub(crate) fn create_persistent_auth_db(base_dir: impl AsRef<Path>) -> DB {
    let key = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09,
        0x08,
    ];
    let opt = rusty_leveldb::Options::new_disk_db_with(key);
    let db_path = base_dir.as_ref().join("authentication_db");
    log::info!("open auth db: {:?}", db_path);
    let database = DB::open(db_path, opt).unwrap();
    database
}

#[cfg(test_mode)]
pub(crate) fn create_in_memory_auth_db(_base_dir: impl AsRef<Path>) -> DB {
    let opt = rusty_leveldb::in_memory();
    log::info!("open in_memory auth db");
    DB::open("authentication_db", opt).unwrap()
}

impl Database {
    pub(crate) fn open(db_base: impl AsRef<Path>) -> Result<Self, DbError> {
        let (sender, receiver) = channel();
        let db_base = db_base.as_ref().to_owned();
        thread::spawn(move || {
            #[cfg(not(test_mode))]
            let mut database = create_persistent_auth_db(&db_base);
            #[cfg(test_mode)]
            let mut database = create_in_memory_auth_db(&db_base);
            loop {
                let call: DBCall = match receiver.recv() {
                    Ok(req) => req,
                    Err(e) => {
                        warn!("mspc receive error: {}", e);
                        break;
                    }
                };
                let sender = call.sender;
                let response = match call.request {
                    DbRequest::Get(request) => match database.get(&request.key) {
                        Some(value) => Ok(DbResponse::Get(GetResponse { value })),
                        None => Err(DbError::UserNotExist),
                    },
                    DbRequest::Delete(request) => match database.delete(&request.key) {
                        Ok(_) => Ok(DbResponse::Delete),
                        Err(_) => Err(DbError::UserNotExist),
                    },
                    DbRequest::Create(request) => match database.get(&request.key) {
                        Some(_) => Err(DbError::UserExist),
                        None => match database.put(&request.key, &request.value) {
                            Ok(_) => match database.flush() {
                                Ok(_) => Ok(DbResponse::Create),
                                Err(_) => Err(DbError::LevelDbInternalError),
                            },
                            Err(_) => Err(DbError::LevelDbInternalError),
                        },
                    },
                    DbRequest::Update(request) => match database.get(&request.key) {
                        Some(_) => match database.put(&request.key, &request.value) {
                            Ok(_) => match database.flush() {
                                Ok(_) => Ok(DbResponse::Update),
                                Err(_) => Err(DbError::LevelDbInternalError),
                            },
                            Err(_) => Err(DbError::LevelDbInternalError),
                        },
                        None => Err(DbError::UserNotExist),
                    },
                    DbRequest::List(request) => match database.new_iter() {
                        Ok(mut iter) => {
                            let mut values = Vec::new();
                            while let Some((_, ref value)) = iter.next() {
                                let user: UserInfo =
                                    serde_json::from_slice(value).unwrap_or_default();
                                if (!request.key.is_empty() && user.has_attribute(&request.key)) || request.key.is_empty() {
                                    values.push(user.id);
                                }
                            }
                            Ok(DbResponse::List(ListResponse { values }))
                        }
                        Err(_) => Err(DbError::LevelDbInternalError),
                    },
                    DbRequest::Ping => Ok(DbResponse::Ping),
                };
                match sender.send(response) {
                    Ok(_) => (),
                    Err(e) => warn!("mpsc send error: {}", e),
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

    pub(crate) fn delete_user(&self, id: &str) -> Result<(), DbError> {
        let (sender, receiver) = channel();
        let request = DbRequest::Delete(DeleteRequest {
            key: id.as_bytes().to_vec(),
        });
        let call = DBCall { sender, request };
        self.sender.send(call)?;
        let result = receiver.recv()?;
        let db_response = result?;
        match db_response {
            DbResponse::Delete => Ok(()),
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

    pub(crate) fn list_users_by_attribute(&self, attribute: &str) -> Result<Vec<String>, DbError> {
        let (sender, receiver) = channel();
        let request = DbRequest::List(ListRequest {
            key: attribute.to_string(),
        });
        let call = DBCall { sender, request };
        self.sender.send(call)?;
        let result = receiver.recv()?;
        let db_response = result?;
        match db_response {
            DbResponse::List(response) => Ok(response.values),
            _ => Err(DbError::UserNotExist),
        }
    }

    pub(crate) fn list_users(&self) -> Result<Vec<String>, DbError> {
        let (sender, receiver) = channel();
        let request = DbRequest::List(ListRequest {
            key: "".to_string(),
        });
        let call = DBCall { sender, request };
        self.sender.send(call)?;
        let result = receiver.recv()?;
        let db_response = result?;
        match db_response {
            DbResponse::List(response) => Ok(response.values),
            _ => Err(DbError::UserNotExist),
        }
    }

    pub(crate) fn update_user(&self, user: &UserInfo) -> Result<(), DbError> {
        let (sender, receiver) = channel();
        let user_bytes = serde_json::to_vec(&user).map_err(|_| DbError::InvalidRequest)?;
        let request = DbRequest::Update(UpdateRequest {
            key: user.id.as_bytes().to_vec(),
            value: user_bytes.to_vec(),
        });
        let call = DBCall { sender, request };
        self.sender.send(call)?;
        let result = receiver.recv()?;
        let db_response = result?;
        match db_response {
            DbResponse::Update => Ok(()),
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
