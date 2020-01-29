use sgx_tunittest::*;
use std::prelude::v1::*;
use teaclave_proto::teaclave_database_service::*;
use teaclave_rpc::endpoint::Endpoint;

pub fn run_tests() {
    rsgx_unit_tests!(
        test_get_success,
        test_get_fail,
        test_put_success,
        test_delete_success,
        test_enqueue_success,
        test_dequeue_success,
        test_dequeue_fail,
    );
}

fn test_get_success() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = GetRequest {
        key: b"test_get_key".to_vec(),
    };
    let response_result = client.get(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());
}

fn test_get_fail() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = GetRequest {
        key: b"test_key_not_exist".to_vec(),
    };
    let response_result = client.get(request);
    assert!(response_result.is_err());
}

fn test_put_success() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = PutRequest {
        key: b"test_put_key".to_vec(),
        value: b"test_put_value".to_vec(),
    };
    let response_result = client.put(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());

    let request = GetRequest {
        key: b"test_put_key".to_vec(),
    };
    let response_result = client.get(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().value, b"test_put_value");
}

fn test_delete_success() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = DeleteRequest {
        key: b"test_delete_key".to_vec(),
    };
    let response_result = client.delete(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());

    let request = GetRequest {
        key: b"test_delete_key".to_vec(),
    };
    let response_result = client.get(request);
    assert!(response_result.is_err());
}

fn test_enqueue_success() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = EnqueueRequest {
        key: b"test_enqueue_key".to_vec(),
        value: b"test_enqueue_value".to_vec(),
    };
    let response_result = client.enqueue(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());
}

fn test_dequeue_success() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = DequeueRequest {
        key: b"test_dequeue_key".to_vec(),
    };
    let response_result = client.dequeue(request);
    assert!(response_result.is_err());
    let request = EnqueueRequest {
        key: b"test_dequeue_key".to_vec(),
        value: b"1".to_vec(),
    };
    let response_result = client.enqueue(request);
    assert!(response_result.is_ok());
    let request = EnqueueRequest {
        key: b"test_dequeue_key".to_vec(),
        value: b"2".to_vec(),
    };
    let response_result = client.enqueue(request);
    assert!(response_result.is_ok());
    let request = DequeueRequest {
        key: b"test_dequeue_key".to_vec(),
    };
    let response_result = client.dequeue(request);
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().value, b"1");
    let request = DequeueRequest {
        key: b"test_dequeue_key".to_vec(),
    };
    let response_result = client.dequeue(request);
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().value, b"2");
}

fn test_dequeue_fail() {
    let channel = Endpoint::new("localhost:7778").connect().unwrap();
    let mut client = TeaclaveDatabaseClient::new(channel).unwrap();
    let request = DequeueRequest {
        key: b"test_dequeue_key".to_vec(),
    };
    let response_result = client.dequeue(request);
    assert!(response_result.is_err());

    let request = EnqueueRequest {
        key: b"test_dequeue_key".to_vec(),
        value: b"1".to_vec(),
    };
    let response_result = client.enqueue(request);
    assert!(response_result.is_ok());
    let request = DequeueRequest {
        key: b"test_dequeue_key".to_vec(),
    };
    let response_result = client.dequeue(request);
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().value, b"1");
    let request = DequeueRequest {
        key: b"test_dequeue_key".to_vec(),
    };
    let response_result = client.dequeue(request);
    assert!(response_result.is_err());
}
