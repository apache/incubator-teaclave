use std::prelude::v1::*;
use teaclave_proto::teaclave_database_service::{
    self, TeaclaveDatabase, GetRequest, GetResponse, PutRequest, PutResponse, DeleteRequest, DeleteResponse, EnqueueRequest, EnqueueResponse,
    DequeueRequest, DequeueResponse,
};
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::{TeaclaveServiceResponseError, TeaclaveServiceResponseResult};
use thiserror::Error;
use rusty_leveldb::DB;
use crate::proxy::ProxyRequest;
use std::cell::RefCell;
use std::sync::mpsc::Receiver;

#[derive(Error, Debug)]
pub enum TeaclaveDatabaseError {
    #[error("key not exist")]
    KeyNotExist,
    #[error("mpsc error")]
    MpscError,
    #[error("leveldb error")]
    LevelDbError,
    #[error("queue empty")]
    QueueEmpty,
}

impl From<TeaclaveDatabaseError> for TeaclaveServiceResponseError {
    fn from(error: TeaclaveDatabaseError) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}

#[teaclave_service(
    teaclave_database_service,
    TeaclaveDatabase,
    TeaclaveDatabaseError
)]
pub struct TeaclaveDatabaseService {
    // LevelDB uses ```&mut self``` in its apis, but the service will use ```&self``` in each request,
    // so we need to wrap the DB with RefCell. 
    // The service is running in a single thread, it's safe to use RefCell
    pub database: RefCell<DB>,
    pub receiver: Receiver<ProxyRequest>,
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
        DBQueue {
            database,
            key, 
        }
    }

    pub fn enqueue(&mut self, value: &[u8]) -> TeaclaveServiceResponseResult<()> {
        let mut tail_index = self.get_tail();
        // put element
        self.database.put(&self.get_element_key(tail_index), value).map_err(|_| TeaclaveDatabaseError::LevelDbError)?;
        // tail + 1
        tail_index += 1;
        self.database.put(&self.get_tail_key(), &tail_index.to_le_bytes()).map_err(|_| TeaclaveDatabaseError::LevelDbError)?;
        Ok(())
    }

    pub fn dequeue(&mut self) -> TeaclaveServiceResponseResult<Vec<u8>> {
        let mut head_index = self.get_head();
        let tail_index = self.get_tail();
        // check whether the queue is empty
        if head_index >= tail_index {
            return Err(TeaclaveDatabaseError::QueueEmpty.into());
        } else {
            let element_key = self.get_element_key(head_index);
            let result = match self.database.get(&element_key) {
                Some(value) => value,
                None => return Err(TeaclaveDatabaseError::LevelDbError.into()),
            };
            // update head
            head_index += 1;
            self.database.put(&self.get_head_key(), &head_index.to_le_bytes()).map_err(|_| TeaclaveDatabaseError::LevelDbError)?;
            // delete element; it's ok to ignore the error
            let _ = self.database.delete(&element_key);
            Ok(result)
        }
    }
}

impl TeaclaveDatabaseService {
    pub fn execution(&mut self) {
        #[cfg(test_mode)]
        test_mode::repalce_with_mock_database(self);
        
        loop {
            let request = match self.receiver.recv() {
                Ok(req) => req,
                Err(e) => { 
                    error!("mspc receive error: {}", e);
                    continue;
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
impl TeaclaveDatabase for TeaclaveDatabaseService {
    fn get(
        &self,
        request: GetRequest,
    ) -> TeaclaveServiceResponseResult<GetResponse> {
        match self.database.borrow_mut().get(&request.key) {
            Some(value) => Ok(GetResponse {
                value: value.to_owned(),
            }),
            None => Err(TeaclaveDatabaseError::KeyNotExist.into())
        }
    }

    fn put(
        &self,
        request: PutRequest,
    ) -> TeaclaveServiceResponseResult<PutResponse> {
        match self.database.borrow_mut().put(&request.key, &request.value) {
            Ok(_) => Ok(PutResponse {}),
            Err(_) => Err(TeaclaveDatabaseError::LevelDbError.into())
        }
    }

    fn delete(
        &self,
        request: DeleteRequest,
    ) -> TeaclaveServiceResponseResult<DeleteResponse> {
        match self.database.borrow_mut().delete(&request.key) {
            Ok(_) => Ok(DeleteResponse {}),
            Err(_) => Err(TeaclaveDatabaseError::LevelDbError.into())
        }
    }

    fn enqueue(
        &self,
        request: EnqueueRequest,
    ) -> TeaclaveServiceResponseResult<EnqueueResponse> {
        let mut db = self.database.borrow_mut();
        let mut queue = DBQueue::open(&mut db, &request.key);
        match queue.enqueue(&request.value) {
            Ok(_) => Ok(EnqueueResponse {}),
            Err(_) => Err(TeaclaveDatabaseError::LevelDbError.into())
        }
    }

    fn dequeue(
        &self,
        request: DequeueRequest,
    ) -> TeaclaveServiceResponseResult<DequeueResponse> {
        let mut db = self.database.borrow_mut();
        let mut queue = DBQueue::open(&mut db, &request.key);
        match queue.dequeue() {
            Ok(value) => Ok(DequeueResponse { value }),
            Err(e) => Err(e)
        }
    }

}

#[cfg(test_mode)]
mod test_mode {
    use super::*;
    pub fn repalce_with_mock_database(service: &mut TeaclaveDatabaseService) {
        let opt = rusty_leveldb::in_memory();
        let mut database = DB::open("mock_db", opt).unwrap();        
        database.put(b"test_get_key", b"test_get_value").unwrap();
        database.put(b"test_delete_key", b"test_delete_value").unwrap();
        service.database.replace(database);
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    fn get_mock_service() -> TeaclaveDatabaseService {
        let (_sender, receiver) = channel();
        let opt = rusty_leveldb::in_memory();
        let mut database = DB::open("mock_db", opt).unwrap();        
        database.put(b"test_get_key", b"test_get_value").unwrap();
        database.put(b"test_delete_key", b"test_delete_value").unwrap();
        TeaclaveDatabaseService {
            database: RefCell::new(database),
            receiver,
        }
    }

    pub fn test_get_key() {
        let service = get_mock_service();
        let request = GetRequest {
            key: b"test_get_key".to_vec(),
        };
        assert!(service.get(request).is_ok());
    }

    pub fn test_put_key() {
        let service = get_mock_service();
        let request = PutRequest {
            key: b"test_put_key".to_vec(),
            value: b"test_put_value".to_vec(),
        };
        assert!(service.put(request).is_ok());
        let request = GetRequest {
            key: b"test_put_key".to_vec(),
        };
        assert!(service.get(request).is_ok());
    }

    pub fn test_delete_key() {
        let service = get_mock_service();
        let request = DeleteRequest {
            key: b"test_delete_key".to_vec(),
        };
        assert!(service.delete(request).is_ok());
        let request = GetRequest {
            key: b"test_delete_key".to_vec(),
        };
        assert!(service.get(request).is_err());
    }

    pub fn test_enqueue() {
        let service = get_mock_service();
        let request = EnqueueRequest {
            key: b"test_enqueue_key".to_vec(),
            value: b"1".to_vec(),
        };
        assert!(service.enqueue(request).is_ok());
        let request = EnqueueRequest {
            key: b"test_enqueue_key".to_vec(),
            value: b"2".to_vec(),
        };
        assert!(service.enqueue(request).is_ok());
    }

    pub fn test_dequeue() {
        let service = get_mock_service();
        let request = DequeueRequest {
            key: b"test_dequeue_key".to_vec(),
        };
        assert!(service.dequeue(request).is_err());
        let request = EnqueueRequest {
            key: b"test_dequeue_key".to_vec(),
            value: b"1".to_vec(),
        };
        assert!(service.enqueue(request).is_ok());
        let request = EnqueueRequest {
            key: b"test_dequeue_key".to_vec(),
            value: b"2".to_vec(),
        };
        assert!(service.enqueue(request).is_ok());
        let request = DequeueRequest {
            key: b"test_dequeue_key".to_vec(),
        };
        assert_eq!(service.dequeue(request).unwrap().value, b"1");
        let request = DequeueRequest {
            key: b"test_dequeue_key".to_vec(),
        };
        assert_eq!(service.dequeue(request).unwrap().value, b"2");
    }
}
