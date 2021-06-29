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

use crate::error::TeaclaveStorageError;
use crate::proxy::ProxyRequest;
use rusty_leveldb::DB;
use std::cell::RefCell;
use std::prelude::v1::*;
use std::sync::mpsc::Receiver;
use teaclave_proto::teaclave_storage_service::{
    DeleteRequest, DeleteResponse, DequeueRequest, DequeueResponse, EnqueueRequest,
    EnqueueResponse, GetRequest, GetResponse, PutRequest, PutResponse, TeaclaveStorage,
};
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::{bail, teaclave_service};
use teaclave_types::TeaclaveServiceResponseResult;

#[teaclave_service(teaclave_storage_service, TeaclaveStorage, TeaclaveStorageError)]
pub(crate) struct TeaclaveStorageService {
    // Current LevelDB implementation is not concurrent, so we need to wrap the
    // DB with RefCell. This service is running in a single thread, it's safe to
    // use RefCell.
    database: RefCell<DB>,
    receiver: Receiver<ProxyRequest>,
}

impl TeaclaveStorageService {
    pub(crate) fn new(database: RefCell<DB>, receiver: Receiver<ProxyRequest>) -> Self {
        Self { database, receiver }
    }
}

// queue-key-head: u32; include element
// queue-key-tail: u32; not include element; if head == tail, queue is empty
// queue-key-index: Vec<u8>; elements
// Todo: what if there are errors when doing get_tail and get_head
struct DBQueue<'a> {
    database: &'a mut DB,
    key: &'a [u8],
}

impl<'a> DBQueue<'a> {
    fn get_tail_key(&self) -> Vec<u8> {
        let mut head_key = b"queue-".to_vec();
        head_key.extend_from_slice(self.key);
        head_key.extend_from_slice(b"-tail");
        head_key
    }
    fn get_head_key(&self) -> Vec<u8> {
        let mut head_key = b"queue-".to_vec();
        head_key.extend_from_slice(self.key);
        head_key.extend_from_slice(b"-head");
        head_key
    }
    fn get_element_key(&self, index: u32) -> Vec<u8> {
        let mut element_key = b"queue-".to_vec();
        element_key.extend_from_slice(self.key);
        element_key.extend_from_slice(b"-");
        element_key.extend_from_slice(&index.to_le_bytes());
        element_key
    }

    fn get_head(&mut self) -> u32 {
        let head_key = self.get_head_key();
        self.read_u32(&head_key).unwrap_or(0)
    }

    fn get_tail(&mut self) -> u32 {
        let tail_key = self.get_tail_key();
        self.read_u32(&tail_key).unwrap_or(0)
    }

    fn read_u32(&mut self, key: &[u8]) -> Option<u32> {
        let element_bytes: Vec<u8> = match self.database.get(key) {
            Some(bytes) => bytes,
            None => return None,
        };
        if element_bytes.len() != 4 {
            return None;
        }
        let mut bytes: [u8; 4] = [0; 4];
        bytes.copy_from_slice(&element_bytes);
        Some(u32::from_le_bytes(bytes))
    }

    pub fn open(database: &'a mut DB, key: &'a [u8]) -> Self {
        DBQueue { database, key }
    }

    pub fn enqueue(&mut self, value: &[u8]) -> TeaclaveServiceResponseResult<()> {
        let tail_index = self.get_tail();
        // put element
        self.database
            .put(&self.get_element_key(tail_index), value)
            .map_err(TeaclaveStorageError::LevelDb)?;

        // update tail
        let tail_index = tail_index.wrapping_add(1);
        self.database
            .put(&self.get_tail_key(), &tail_index.to_le_bytes())
            .map_err(TeaclaveStorageError::LevelDb)?;
        Ok(())
    }

    pub fn dequeue(&mut self) -> TeaclaveServiceResponseResult<Vec<u8>> {
        let head_index = self.get_head();
        let tail_index = self.get_tail();
        // check whether the queue is empty
        if head_index == tail_index {
            Err(TeaclaveStorageError::None.into())
        } else {
            let element_key = self.get_element_key(head_index);
            let result = match self.database.get(&element_key) {
                Some(value) => value,
                None => bail!(TeaclaveStorageError::None),
            };

            // update head
            let head_index = head_index.wrapping_add(1);
            self.database
                .put(&self.get_head_key(), &head_index.to_le_bytes())
                .map_err(TeaclaveStorageError::LevelDb)?;
            // delete element; it's ok to ignore the error
            let _ = self.database.delete(&element_key);
            Ok(result)
        }
    }

    #[allow(unused)]
    pub fn len(&mut self) -> u32 {
        let head_index = self.get_head();
        let tail_index = self.get_tail();

        if tail_index >= head_index {
            tail_index - head_index
        } else {
            u32::MAX - head_index + tail_index + 1
        }
    }
}

impl TeaclaveStorageService {
    pub(crate) fn start(&mut self) {
        #[cfg(test_mode)]
        test_mode::repalce_with_mock_database(self);

        loop {
            let request = match self.receiver.recv() {
                Ok(req) => req,
                Err(e) => {
                    error!("mspc receive error: {}", e);
                    break;
                }
            };
            let database_request = request.request;
            let sender = request.sender;
            let response = self.dispatch(database_request);

            match sender.send(response) {
                Ok(_) => (),
                Err(e) => error!("mpsc send error: {}", e),
            }
        }
    }
}
impl TeaclaveStorage for TeaclaveStorageService {
    fn get(&self, request: Request<GetRequest>) -> TeaclaveServiceResponseResult<GetResponse> {
        let request = request.message;
        match self.database.borrow_mut().get(&request.key) {
            Some(value) => Ok(GetResponse { value }),
            None => Err(TeaclaveStorageError::None.into()),
        }
    }

    fn put(&self, request: Request<PutRequest>) -> TeaclaveServiceResponseResult<PutResponse> {
        let request = request.message;
        self.database
            .borrow_mut()
            .put(&request.key, &request.value)
            .map_err(TeaclaveStorageError::LevelDb)?;
        Ok(PutResponse)
    }

    fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> TeaclaveServiceResponseResult<DeleteResponse> {
        let request = request.message;
        self.database
            .borrow_mut()
            .delete(&request.key)
            .map_err(TeaclaveStorageError::LevelDb)?;
        Ok(DeleteResponse)
    }

    fn enqueue(
        &self,
        request: Request<EnqueueRequest>,
    ) -> TeaclaveServiceResponseResult<EnqueueResponse> {
        let request = request.message;
        let mut db = self.database.borrow_mut();
        let mut queue = DBQueue::open(&mut db, &request.key);
        queue.enqueue(&request.value).map(|_| EnqueueResponse)
    }

    fn dequeue(
        &self,
        request: Request<DequeueRequest>,
    ) -> TeaclaveServiceResponseResult<DequeueResponse> {
        let request = request.message;
        let mut db = self.database.borrow_mut();
        let mut queue = DBQueue::open(&mut db, &request.key);
        queue.dequeue().map(|value| DequeueResponse { value })
    }
}

#[cfg(test_mode)]
mod test_mode {
    use super::*;
    pub(crate) fn repalce_with_mock_database(service: &mut TeaclaveStorageService) {
        let opt = rusty_leveldb::in_memory();
        let mut database = DB::open("mock_db", opt).unwrap();
        database.put(b"test_get_key", b"test_get_value").unwrap();
        database
            .put(b"test_delete_key", b"test_delete_value")
            .unwrap();
        service.database.replace(database);
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::sync::mpsc::channel;
    use teaclave_rpc::IntoRequest;

    fn get_mock_service() -> TeaclaveStorageService {
        let (_sender, receiver) = channel();
        let opt = rusty_leveldb::in_memory();
        let mut database = DB::open("mock_db", opt).unwrap();
        database.put(b"test_get_key", b"test_get_value").unwrap();
        database
            .put(b"test_delete_key", b"test_delete_value")
            .unwrap();
        TeaclaveStorageService {
            database: RefCell::new(database),
            receiver,
        }
    }

    pub fn test_get_key() {
        let service = get_mock_service();
        let request = GetRequest::new("test_get_key").into_request();
        assert!(service.get(request).is_ok());
    }

    pub fn test_put_key() {
        let service = get_mock_service();
        let request = PutRequest::new("test_put_key", "test_put_value").into_request();
        assert!(service.put(request).is_ok());
        let request = GetRequest::new("test_put_key").into_request();
        assert!(service.get(request).is_ok());
    }

    pub fn test_delete_key() {
        let service = get_mock_service();
        let request = DeleteRequest::new("test_delete_key").into_request();
        assert!(service.delete(request).is_ok());
        let request = GetRequest::new("test_delete_key").into_request();
        assert!(service.get(request).is_err());
    }

    pub fn test_enqueue() {
        let service = get_mock_service();
        let request = EnqueueRequest::new("test_enqueue_key", "1").into_request();
        assert!(service.enqueue(request).is_ok());
        let request = EnqueueRequest::new("test_enqueue_key", "2").into_request();
        assert!(service.enqueue(request).is_ok());
    }

    pub fn test_dequeue() {
        let service = get_mock_service();
        let request = DequeueRequest::new("test_dequeue_key").into_request();
        assert!(service.dequeue(request).is_err());
        let request = EnqueueRequest::new("test_dequeue_key", "1").into_request();
        assert!(service.enqueue(request).is_ok());
        let request = EnqueueRequest::new("test_dequeue_key", "2").into_request();
        assert!(service.enqueue(request).is_ok());
        let request = DequeueRequest::new("test_dequeue_key").into_request();
        assert_eq!(service.dequeue(request).unwrap().value, b"1");
        let request = DequeueRequest::new("test_dequeue_key").into_request();
        assert_eq!(service.dequeue(request).unwrap().value, b"2");
    }
}
