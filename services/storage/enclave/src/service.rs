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

use crate::error::StorageServiceError;
use crate::proxy::ProxyRequest;
use anyhow::anyhow;
use rusty_leveldb::LdbIterator;
use rusty_leveldb::DB;
use std::cell::RefCell;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_service_enclave_utils::bail;
use tokio::sync::mpsc::UnboundedReceiver;

pub(crate) struct TeaclaveStorageService {
    // Current LevelDB implementation is not concurrent, so we need to wrap the
    // DB with RefCell. This service is running in a single thread, it's safe to
    // use RefCell.
    database: RefCell<DB>,
    receiver: UnboundedReceiver<ProxyRequest>,
}

impl TeaclaveStorageService {
    pub(crate) fn new(database: RefCell<DB>, receiver: UnboundedReceiver<ProxyRequest>) -> Self {
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

    pub fn enqueue(&mut self, value: &[u8]) -> Result<(), StorageServiceError> {
        let tail_index = self.get_tail();
        // put element
        self.database
            .put(&self.get_element_key(tail_index), value)?;

        // update tail
        let tail_index = tail_index.wrapping_add(1);
        self.database
            .put(&self.get_tail_key(), &tail_index.to_le_bytes())?;

        self.database.flush()?;
        Ok(())
    }

    pub fn dequeue(&mut self) -> Result<Vec<u8>, StorageServiceError> {
        let head_index = self.get_head();
        let tail_index = self.get_tail();
        // check whether the queue is empty
        if head_index == tail_index {
            bail!(StorageServiceError::Service(anyhow!(
                "head_index == tail_index"
            )))
        } else {
            let element_key = self.get_element_key(head_index);
            let result = match self.database.get(&element_key) {
                Some(value) => value,
                None => bail!(StorageServiceError::Service(anyhow!(
                    "cannot get element_key"
                ))),
            };

            // update head
            let head_index = head_index.wrapping_add(1);
            self.database
                .put(&self.get_head_key(), &head_index.to_le_bytes())?;
            self.database.delete(&element_key)?;
            self.database.compact_range(b"queue", b"queuf")?;
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
        while let Some(request) = self.receiver.blocking_recv() {
            let database_request = request.request;
            let sender = request.sender;
            let response = self.dispatch(database_request);

            match sender.send(response) {
                Ok(_) => (),
                Err(e) => error!("mpsc send error: {}", e),
            }
        }
    }

    fn dispatch(
        &self,
        request: teaclave_rpc::Request<TeaclaveStorageRequest>,
    ) -> std::result::Result<TeaclaveStorageResponse, StorageServiceError> {
        match request.into_inner() {
            TeaclaveStorageRequest::Get(r) => {
                let response = self.get(r)?;
                Ok(response).map(TeaclaveStorageResponse::Get)
            }
            TeaclaveStorageRequest::Put(r) => {
                let response = self.put(r)?;
                Ok(response).map(TeaclaveStorageResponse::Put)
            }
            TeaclaveStorageRequest::Delete(r) => {
                let response = self.delete(r)?;
                Ok(response).map(TeaclaveStorageResponse::Delete)
            }
            TeaclaveStorageRequest::Enqueue(r) => {
                let response = self.enqueue(r)?;
                Ok(response).map(TeaclaveStorageResponse::Enqueue)
            }
            TeaclaveStorageRequest::Dequeue(r) => {
                let response = self.dequeue(r)?;
                Ok(response).map(TeaclaveStorageResponse::Dequeue)
            }
            TeaclaveStorageRequest::GetKeysByPrefix(r) => {
                let response = self.get_keys_by_prefix(r)?;
                Ok(response).map(TeaclaveStorageResponse::GetKeysByPrefix)
            }
        }
    }
}

impl TeaclaveStorageService {
    fn get(&self, request: GetRequest) -> std::result::Result<GetResponse, StorageServiceError> {
        match self.database.borrow_mut().get(&request.key) {
            Some(value) => Ok(GetResponse { value }),
            None => bail!(StorageServiceError::None),
        }
    }

    fn put(&self, request: PutRequest) -> std::result::Result<PutResponse, StorageServiceError> {
        self.database
            .borrow_mut()
            .put(&request.key, &request.value)
            .map_err(StorageServiceError::Database)?;

        self.database
            .borrow_mut()
            .flush()
            .map_err(StorageServiceError::Database)?;
        Ok(PutResponse {})
    }

    fn delete(
        &self,
        request: DeleteRequest,
    ) -> std::result::Result<DeleteResponse, StorageServiceError> {
        self.database
            .borrow_mut()
            .delete(&request.key)
            .map_err(StorageServiceError::Database)?;

        self.database
            .borrow_mut()
            .flush()
            .map_err(StorageServiceError::Database)?;
        Ok(DeleteResponse {})
    }

    fn enqueue(
        &self,
        request: EnqueueRequest,
    ) -> std::result::Result<EnqueueResponse, StorageServiceError> {
        let mut db = self.database.borrow_mut();
        let mut queue = DBQueue::open(&mut db, &request.key);
        match queue.enqueue(&request.value) {
            Ok(_) => Ok(EnqueueResponse {}),
            Err(e) => bail!(e),
        }
    }

    fn dequeue(
        &self,
        request: DequeueRequest,
    ) -> std::result::Result<DequeueResponse, StorageServiceError> {
        let mut db = self.database.borrow_mut();
        let mut queue = DBQueue::open(&mut db, &request.key);
        match queue.dequeue() {
            Ok(value) => Ok(DequeueResponse { value }),
            Err(e) => bail!(e),
        }
    }

    fn get_keys_by_prefix(
        &self,
        request: GetKeysByPrefixRequest,
    ) -> std::result::Result<GetKeysByPrefixResponse, StorageServiceError> {
        let prefix = request.prefix;
        let mut db = self.database.borrow_mut();
        let mut it = db.new_iter().map_err(StorageServiceError::Database)?;

        let mut first_prefix = prefix.clone();
        first_prefix.push(b'-');
        let mut last_prefix = prefix;
        last_prefix.push(b'.');

        it.seek(&first_prefix[..]);
        if !it.valid() {
            return Ok(GetKeysByPrefixResponse::default());
        }
        let mut key = Vec::new();
        let mut value = Vec::new();
        let mut keys = Vec::new();
        if !it.current(&mut key, &mut value) {
            return Ok(GetKeysByPrefixResponse::default());
        }
        keys.push(key);

        while let Some((k, _)) = it.next() {
            if k >= last_prefix {
                break;
            }
            keys.push(k);
        }

        Ok(GetKeysByPrefixResponse { keys })
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;

    fn get_mock_service() -> TeaclaveStorageService {
        let (_sender, receiver) = unbounded_channel();
        let key = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a,
            0x09, 0x08,
        ];
        let opt = rusty_leveldb::Options::new_disk_db_with(key);
        let mut database = DB::open("mock_db_unit_test", opt).unwrap();
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
        let request = GetRequest::new("test_get_key");
        assert!(service.get(request).is_ok());
    }

    pub fn test_put_key() {
        let service = get_mock_service();
        let request = PutRequest::new("test_put_key", "test_put_value");
        assert!(service.put(request).is_ok());
        let request = GetRequest::new("test_put_key");
        assert!(service.get(request).is_ok());
    }

    pub fn test_delete_key() {
        let service = get_mock_service();
        let request = DeleteRequest::new("test_delete_key");
        assert!(service.delete(request).is_ok());
        let request = GetRequest::new("test_delete_key");
        assert!(service.get(request).is_err());
    }

    pub fn test_enqueue() {
        let service = get_mock_service();
        let request = EnqueueRequest::new("test_enqueue_key", "1");
        assert!(service.enqueue(request).is_ok());
        let request = EnqueueRequest::new("test_enqueue_key", "2");
        assert!(service.enqueue(request).is_ok());
    }

    pub fn test_dequeue() {
        let service = get_mock_service();
        let request = DequeueRequest::new("test_dequeue_key");
        assert!(service.dequeue(request).is_err());
        let request = EnqueueRequest::new("test_dequeue_key", "1");
        assert!(service.enqueue(request).is_ok());
        let request = EnqueueRequest::new("test_dequeue_key", "2");
        assert!(service.enqueue(request).is_ok());
        let request = DequeueRequest::new("test_dequeue_key");
        assert_eq!(service.dequeue(request).unwrap().value, b"1");
        let request = DequeueRequest::new("test_dequeue_key");
        assert_eq!(service.dequeue(request).unwrap().value, b"2");
    }

    pub fn test_get_keys_by_prefix() {
        let service = get_mock_service();
        let request = PutRequest::new("function-1", "test_put_value");
        assert!(service.put(request).is_ok());
        let request = PutRequest::new("function-22", "test_put_value");
        assert!(service.put(request).is_ok());
        let request = PutRequest::new("function-333", "test_put_value");
        assert!(service.put(request).is_ok());
        let request = PutRequest::new("task-444", "test_put_value");
        assert!(service.put(request).is_ok());
        let request = PutRequest::new("function-5", "test_put_value");
        assert!(service.put(request).is_ok());
        let request = GetKeysByPrefixRequest::new("function");
        let response = service.get_keys_by_prefix(request);
        assert!(response.is_ok());
        assert_eq!(
            response.unwrap().keys,
            std::vec![
                b"function-1".to_vec(),
                b"function-22".to_vec(),
                b"function-333".to_vec(),
                b"function-5".to_vec()
            ]
        );
    }
}
