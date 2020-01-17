use sgx_tunittest::*;
use std::prelude::v1::*;
use teaclave_proto::teaclave_database_service::*;
use teaclave_rpc::endpoint::Endpoint;

pub fn run_functional_tests() {
    rsgx_unit_tests!(
        test_get_successful,
        test_get_failed,
        test_put_successful,
        test_delete_successful,
        test_enqueue_successful,
        test_dequeue_successful,
        test_dequeue_failed,
    );
}

fn test_get_successful() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = GetRequest {
        key: b"test_get_key".to_vec(),
    }
    .into();
    let response_result = client.get(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());
}

fn test_get_failed() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = GetRequest {
        key: b"test_key_not_exist".to_vec(),
    }
    .into();
    let response_result = client.get(request);
    assert!(response_result.is_err());
}

fn test_put_successful() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = PutRequest {
        key: b"test_put_key".to_vec(),
        value: b"test_put_value".to_vec(),
    }
    .into();
    let response_result = client.put(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());

    let request = GetRequest {
        key: b"test_put_key".to_vec(),
    }
    .into();
    let response_result = client.get(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().value, b"test_put_value");
}

fn test_delete_successful() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = DeleteRequest {
        key: b"test_delete_key".to_vec(),
    }
    .into();
    let response_result = client.delete(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());

    let request = GetRequest {
        key: b"test_delete_key".to_vec(),
    }
    .into();
    let response_result = client.get(request);
    assert!(response_result.is_err());
}

fn test_enqueue_successful() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = EnqueueRequest {
        key: b"test_enqueue_key".to_vec(),
        value: b"test_enqueue_value".to_vec(),
    }
    .into();
    let response_result = client.enqueue(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());
}

fn test_dequeue_successful() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = DequeueRequest {
        key: b"test_dequeue_key".to_vec(),
    }
    .into();
    let response_result = client.dequeue(request);
    assert!(response_result.is_err());
    let request = EnqueueRequest {
        key: b"test_dequeue_key".to_vec(),
        value: b"1".to_vec(),
    }
    .into();
    let response_result = client.enqueue(request);
    assert!(response_result.is_ok());
    let request = EnqueueRequest {
        key: b"test_dequeue_key".to_vec(),
        value: b"2".to_vec(),
    }
    .into();
    let response_result = client.enqueue(request);
    assert!(response_result.is_ok());
    let request = DequeueRequest {
        key: b"test_dequeue_key".to_vec(),
    }
    .into();
    let response_result = client.dequeue(request);
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().value, b"1");
    let request = DequeueRequest {
        key: b"test_dequeue_key".to_vec(),
    }
    .into();
    let response_result = client.dequeue(request);
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().value, b"2");
}

fn test_dequeue_failed() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = DequeueRequest {
        key: b"test_dequeue_key".to_vec(),
    }
    .into();
    let response_result = client.dequeue(request);
    assert!(response_result.is_err());

    let request = EnqueueRequest {
        key: b"test_dequeue_key".to_vec(),
        value: b"1".to_vec(),
    }
    .into();
    let response_result = client.enqueue(request);
    assert!(response_result.is_ok());
    let request = DequeueRequest {
        key: b"test_dequeue_key".to_vec(),
    }
    .into();
    let response_result = client.dequeue(request);
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().value, b"1");
    let request = DequeueRequest {
        key: b"test_dequeue_key".to_vec(),
    }
    .into();
    let response_result = client.dequeue(request);
    assert!(response_result.is_err());
}
