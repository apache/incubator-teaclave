# Task Management Service

## External API (port: 5554)

### Create Task
#### Create Task request data structure.
```rust
#[derive(Serialize)]
pub struct CreateTaskRequest {
    pub function_name: String,
    pub collaborator_list: Vec<String>,
    pub files: Vec<String>,
    pub user_id: String,
    pub user_token: String,
}
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum TaskRequest {
    Create(CreateTaskRequest),
}
```
#### Request examples

echo :
```json
{
  "type":"Create",
  "function_name":"echo",
  "collaborator_list":[],
  "files":[],
  "user_id":"user1",
  "user_token":"token1"
}
```
psi: 
``` json
{
  "type":"Create",
  "function_name":"psi",
  "collaborator_list":["user2"],
  "files":["0a9ca5ac-2150-4bdd-ab63-c2c12252e747"],
  "user_id":"user1",
  "user_token":"token1"
}
```
#### Create Task response data structure
``` rust
#[derive(Deserialize)]
pub struct CreateTaskResponse {
    pub task_id: String,
    pub task_token: String,
    pub ip: IpAddr,
    pub port: u16,
}
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum TaskResponse {
    Create(CreateTaskResponse),
}
```
#### Response examples:

echo:
``` json
{
  "type":"Create",
  "task_id":"b77f124b-f790-4af5-93de-cd1ec8946c25",
  "task_token":"53c5a8c6e26e4df83987070d3c865318",
  "ip":"127.0.0.1",
  "port":3444
}
```
psi:
``` json
{
  "type":"Create",
  "task_id":"5ef0cfc7-11e9-445a-8c46-23790ea86819",
  "task_token":"70a58efaa0dc3a567f1c2bc3555d95b8",
  "ip":"127.0.0.1",
  "port":3444
}
```

### Get Task
#### Get Task request data structure: 
```rust
#[derive(Serialize)]
pub struct GetTaskRequest {
    pub task_id: String,
    pub user_id: String,
    pub user_token: String,
}
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum TaskRequest {
    Get(GetTaskRequest),
}
```



#### Request examples:
```json
{
	"type":"Get",
	"task_id":"f1b0e2a2-865f-11e9-8003-010203040506",
	"user_id":"bbbb",
	"user_token":"xxxx"
}
```

#### Get Task response data structure
```rust
#[derive(Deserialize)]
pub enum FunctionType {
    Single,
    Multiparty,
}
#[derive(Deserialize)]
pub enum TaskStatus {
    Created,
    Ready,
    Running,
    Finished,
    Failed,
}
#[derive(Deserialize)]
pub struct CollaboratorStatus {
    pub user_id: String,
    pub approved: bool,
}
#[derive(Deserialize)]
pub struct TaskInfo {
    pub user_id: String,
    pub function_name: String,
    pub function_type: FunctionType,
    pub status: TaskStatus,
    pub ip: IpAddr,
    pub port: u16,
    pub task_token: String,
    pub collaborator_list: Vec<CollaboratorStatus>,
    pub task_result_file_id: Option<String>,
    pub user_private_result_file_id: Vec<String>,
}
#[derive(Deserialize)]
pub struct GetTaskResponse {
    pub task_info: TaskInfo,
}
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum TaskResponse {
    Get(GetTaskResponse),
}
```
#### Response examples:

echo task:
``` json
{
  "type":"Get",
  "task_info":
  {
    "user_id":"user1",
    "function_name":"echo",
    "function_type":"Single",
    "status":"Ready",
    "ip":"127.0.0.1",
    "port":3444,
    "task_token":"3328ce7d42c1f5ebe2660926397ac934",
    "collaborator_list":[],
    "task_result_file_id":null,
    "user_private_result_file_id":[]
  }
}
```
psi task
``` json
{
  "type":"Get",
  "task_info":{
    "user_id":"user1",
    "function_name":"psi",
    "function_type":"Multiparty",
    "status":"Created",
    "ip":"127.0.0.1",
    "port":3444,
    "task_token":"70a58efaa0dc3a567f1c2b3555d95b8",
    "collaborator_list":[{"user_id":"user2","approved":false}],
    "task_result_file_id":null,
    "user_private_result_file_id":[]
  }
}
```

### UpdateTask (Collaborator approves task and provides files)
#### Update Task Request
```rust
#[derive(Serialize)]
pub struct UpdateTaskRequest {
    pub task_id: String,
    pub files: Vec<String>,
    pub user_id: String,
    pub user_token: String,
}
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum TaskRequest {
    Update(UpdateTaskRequest),
}
```
#### Request example:

```json
{
  "type":"Update",
  "task_id":"5ef0cfc7-11e9-445a-8c46-23790ea86819",
  "files":["b76f9ee2-63d2-43bd-a722-466792af7035"],
  "user_id":"user2","user_token":"token2"
}
```
#### Update Task Response
```rust
#[derive(Deserialize)]
pub enum TaskStatus {
    Created,
    Ready,
    Running,
    Finished,
    Failed,
}
#[derive(Deserialize)]
pub struct UpdateTaskResponse {
    pub success: bool,
    pub status: TaskStatus,
    pub ip: IpAddr,
    pub port: u16,
    pub task_token: String,
}
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum TaskResponse {
    Update(UpdateTaskResponse),
}
```
#### Response example:
```json
{
  "type":"Update",
  "success":true,
  "status":"Ready",
  "ip":"127.0.0.1",
  "port":3444,
  "task_token":"70a58efaa0dc3a567f1c2bc3555d95b8"
}
```
## Internal API (port: 5555)

### Get Task
#### Get Task request data structure
```rust
#[derive(Serialize)]
pub struct GetTaskRequest {
    pub task_id: String,
}
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum TaskRequest {
    Get(GetTaskRequest),
}
```



#### Request examples:
```json
{
	"type":"Get",
	"task_id":"88a6b759-8662-11e9-8001-010203040506"
}
```

#### Get Task response data structure
```rust
#[derive(Deserialize)]
pub enum FunctionType {
    Single,
    Multiparty,
}
#[derive(Deserialize)]
pub enum TaskStatus {
    Created,
    Ready,
    Running,
    Finished,
    Failed,
}
#[derive(Deserialize)]
pub struct CollaboratorStatus {
    pub user_id: String,
    pub approved: bool,
}
#[derive(Deserialize)]
pub struct TaskInfo {
    pub user_id: String,
    pub collaborator_list: Vec<CollaboratorStatus>,
    pub approved_user_number: usize,
    pub function_name: String,
    pub function_type: FunctionType,
    pub status: TaskStatus,
    pub ip: IpAddr,
    pub port: u16,
    pub task_token: String,
    pub input_files: Vec<TaskFile>,
    pub output_files: Vec<TaskFile>,
    pub task_result_file_id: Option<String>,
}
#[derive(Deserialize)]
pub struct GetTaskResponse {
    pub task_info: TaskInfo,
}
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum TaskResponse {
    Get(GetTaskResponse),
}
```
#### Response examples:
```json 
{
  "type":"Get",
  "task_info":{
    "user_id":"user1",
    "collaborator_list":[{"user_id":"user2","approved":true}],
    "approved_user_number":1,
    "function_name":"psi",
    "function_type":"Multiparty",
    "status":"Ready",
    "ip":"127.0.0.1",
    "port":3444,
    "task_token":"70a58efaa0dc3a567f1c2bc3555d95b8",
    "input_files":[{"user_id":"user1","file_id":"0a9ca5ac-2150-4bdd-ab63-c2c12252e747"},
                   {"user_id":"user2","file_id":"b76f9ee2-63d2-43bd-a722-466792af7035"}],
    "output_files":[],
    "task_result_file_id":null
  }
}
```
### Update Task 
#### Update Task request data structure:
```rust
#[derive(Deserialize)]
pub enum TaskStatus {
    Created,
    Ready,
    Running,
    Finished,
    Failed,
}
#[derive(Serialize)]
pub struct UpdateTaskRequest {
    pub task_id: String,
    pub task_result_file_id: Option<String>,
    pub output_files: Vec<TaskFile>,
    pub status: Option<TaskStatus>,
}
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum TaskRequest {
    Update(UpdateTaskRequest),
}
```
#### Request examples:
```json
{
  "type":"Update",
  "task_id":"5ef0cfc7-11e9-445a-8c4623790ea86819",
  "task_result_file_id":null,
  "output_files":[{"user_id":"user1","file_id":"d18fb288-8fef-48c3-b800-201b2734882e"},
                  {"user_id":"user2","file_id":"0e8446d1-6fa5-49db-b828-8087a0141370"}],
  "status":"Finished"
}
```
#### Update Task response data structure
```rust
#[derive(Deserialize)]
pub struct UpdateTaskResponse {
    pub success: bool,
}
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum TaskResponse {
    Update(UpdateTaskResponse),
}
```
#### Response examples:
```json
{
	"type":"Update",
	"success":true
} 
```

# Function Node Service

## External API (port:3444)

### Invoke Task

#### Invoke Task request data structure

```rust
#[derive(Serialize)]
pub struct InvokeTaskRequest {
    pub task_id: String,
    pub function_name: String,
    pub task_token: String,
    pub payload: Option<String>,
}
```

#### Invoke Task request examples

```json
{
  "task_id":"b77f124b-f790-4af5-93de-cd1ec8946c25",
  "function_name":"echo",
  "task_token":"53c5a8c6e26e4df83987070d3c865318",
  "payload":"echo_payload"
}
```

#### 

#### Invoke Task response data structure

```rust
#[derive(Deserialize)]
pub struct InvokeTaskResponse {
    pub result: String,
}
```

#### Response examples:

```json
{
  "result":"echo_payload"
}
```

# KMS

## Enclave API (port: 6016)

### Create Key

#### Create Key request data structure

```rust
#[derive(Serialize)]
pub struct CreateKeyRequest {}
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum KMSRequest {
    Create(CreateKeyRequest),
}
```

#### Create key request examples

```json
{
	"type": "Create"
}
```

#### Create key response data structure

```rust
#[derive(Clone, Deserialize)]
pub struct AEADKeyConfig {
    #[serde(with = "base64_decoder")]
    pub key: Vec<u8>,
    #[serde(with = "base64_decoder")]
    pub nonce: Vec<u8>,
    #[serde(with = "base64_decoder")]
    pub ad: Vec<u8>,
}
#[derive(Deserialize)]
pub struct CreateKeyResponse {
    pub key_id: String,
    pub config: AEADKeyConfig,
}
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum KMSResponse {
    Create(CreateKeyResponse),
}
```

#### response example

```json
{
  "type":"Create",
  "key_id":"cfbe341e-cf3c-463e-a493-6b22c57f51d4",
  "config":{
    "key":"E99iifHZ16dDY0IocaGFyD3v2j/kvHsn6wRqFZWGeDA=",
    "nonce":"jcqPUkPIqgYeLe9B",
    "ad":"V/gZjZY="
  }
}
```

### Get Key

#### Get Key request data structure

```rust
#[derive(Serialize)]
pub struct GetKeyRequest {
    pub key_id: String,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum KMSRequest {
    Get(GetKeyRequest),
}
```

#### Get key request examples

```json
{
	"type":"Get",
	"key_id":"c26ada76-8557-11e9-8002-020203040506"
}
```

#### Get key response data structure

```rust
#[derive(Clone, Deserialize)]
pub struct AEADKeyConfig {
    #[serde(with = "base64_decoder")]
    pub key: Vec<u8>,
    #[serde(with = "base64_decoder")]
    pub nonce: Vec<u8>,
    #[serde(with = "base64_decoder")]
    pub ad: Vec<u8>,
}
#[derive(Deserialize)]
pub struct GetKeyResponse {
    pub config: AEADKeyConfig,
}
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum KMSResponse {
    Get(GetKeyResponse),
}
```

#### response example

```json
{
  "type":"Get",
  "config":{
    "key":"CvEH2KdKLyTRaycAbx28oU5FSUxTEaA9q8gGC6/IwS4=",
    "nonce":"NygUVqPwxJDffni3",
    "ad":"IFx5qXQ="
  }
}
```

#Trusted DFS

## External API (port: 5065)

### Create File

#### Create File request data structure

```rust
#[derive(Serialize)]
pub struct CreateFileRequest {
    pub file_name: String,
    pub sha256: String,
    pub file_size: u32,
    pub user_id: String,
    pub user_token: String,
}
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum DFSRequest {
    Create(CreateFileRequest),
}
```

#### Create File request examples

```json
{
  "type":"Create",
  "file_name":"file",
  "sha256":"233b51fc70dcc0c0ecb4e6ed9dc32d4c421a54859d069ec87fa836cf8c8aeb44",
  "file_size":128,
  "user_id":"user1",
  "user_token":"token1"
}
```

#### Create File response data structure

```rust
#[derive(Clone, Deserialize)]
pub struct AEADKeyConfig {
    #[serde(with = "base64_decoder")]
    pub key: Vec<u8>,
    #[serde(with = "base64_decoder")]
    pub nonce: Vec<u8>,
    #[serde(with = "base64_decoder")]
    pub ad: Vec<u8>,
}
#[derive(Deserialize)]
pub struct CreateFileResponse {
    pub file_id: String,
    pub access_path: String,
    pub key_config: AEADKeyConfig,
}
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum DFSResponse {
    Create(CreateFileResponse),
}
```

#### response example

```json
{
  "type":"Create",
  "file_id":"0a9ca5ac-2150-4bdd-ab63-c2c12252e747",
  "access_path":"/tmp/0a9ca5ac-2150-4bdd-ab63-c2c12252e747",
  "key_config":{
    "key":"cPGcaSWCGDxGshC9FSvOdW/5zal9FmtoNojlxL8Hfqs=",
    "nonce":"cdfFk73Xv9x6YUL8",
    "ad":"IpDKtR4="
  }
}
```

### Get File

#### Get File request data structure

```rust
#[derive(Serialize)]
pub struct GetFileRequest {
    pub file_id: String,
    pub user_id: String,
    pub user_token: String,
}
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum DFSRequest {
    Get(GetFileRequest),
}
```

#### Get File request examples

```json
{
  "type":"Get",
  "file_id":"d18fb288-8fef-48c3-b800-201b2734882e",
  "user_id":"user1",
  "user_token":"token1"
}
```

#### Get File response data structure

```rust
#[derive(Clone, Deserialize)]
pub struct AEADKeyConfig {
    #[serde(with = "base64_decoder")]
    pub key: Vec<u8>,
    #[serde(with = "base64_decoder")]
    pub nonce: Vec<u8>,
    #[serde(with = "base64_decoder")]
    pub ad: Vec<u8>,
}
#[derive(Deserialize)]
pub struct FileInfo {
    pub user_id: String,
    pub file_name: String,
    pub sha256: String,
    pub file_size: u32,
    pub access_path: String,
    pub task_id: Option<String>,
    pub collaborator_list: Vec<String>,
    pub key_config: AEADKeyConfig,
}
#[derive(Deserialize)]
pub struct GetFileResponse {
    pub file_info: FileInfo,
}
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum DFSResponse {
    Get(GetFileResponse),
}
```

#### response example

```json
{
  "type":"Get",
  "file_info":{
    "user_id":"user1",
    "file_name":"5ef0cfc7-11e9-445a-8c46-23790ea86819",
    "sha256":"d5e2d2ac07b741be58f6b9e50ede5fdcf16f3e8053ecef9350e7744b0d8bd90c",
    "file_size":4,
    "access_path":"/tmp/d18fb288-8fef-48c3-b800-201b2734882e",
    "task_id":"5ef0cfc7-11e9-445a-8c46-23790ea86819",
    "collaborator_list":[],
    "key_config":{
      "key":"E99iifHZ16dDY0IocaGFyD3v2j/kvHsn6wRqFZWGeDA=",
      "nonce":"jcqPUkPIqgYeLe9B",
      "ad":"V/gZjZY="
    }
  }
}
```

## Internal API (port: 5066)

### Create File

#### Create File request data structure

```rust
#[derive(Serialize)]
pub struct CreateFileRequest {
    pub sha256: String,
    pub file_size: u32,
    pub user_id: String,
    pub task_id: String,
    pub collaborator_list: Vec<String>,
    pub allow_policy: u32,
}
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum DFSRequest {
    Create(CreateFileRequest),
}
```

#### Create File request examples

```json
{
  "type":"Create",
  "sha256":"85f90dfea1d8027e1463e5ca971a250110a20df0119d204a74220bc63516d15b",
  "file_size":3,
  "user_id":"user2",
  "task_id":"5ef0cfc7-11e9-445a-8c46-23790ea86819",
  "collaborator_list":[],
  "allow_policy":0
}
```

#### Create File response data structure

```rust
#[derive(Clone, Deserialize)]
pub struct AEADKeyConfig {
    #[serde(with = "base64_decoder")]
    pub key: Vec<u8>,
    #[serde(with = "base64_decoder")]
    pub nonce: Vec<u8>,
    #[serde(with = "base64_decoder")]
    pub ad: Vec<u8>,
}
#[derive(Deserialize)]
pub struct CreateFileResponse {
    pub file_id: String,
    pub access_path: String,
    pub key_config: AEADKeyConfig,
}
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum DFSResponse {
    Create(CreateFileResponse),
}
```

#### response example

```json
{
  "type":"Create",
  "file_id":"0e8446d1-6fa5-49db-b828-8087a0141370",
  "access_path":"/tmp/0e8446d1-6fa5-49db-b828-8087a0141370",
  "key_config":{
    "key":"UKGHBJyhOCKHjbbHFOpo+WNJymG8KcmnrfyL/zs8IOY=",
    "nonce":"5fahqAqvPrhYg0l9",
    "ad":"unSzGpc="
  }
}
```

### Get File

#### Get File request data structure

```rust
#[derive(Serialize)]
pub struct GetFileRequest {
    pub file_id: String,
}
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum DFSRequest {
    Get(GetFileRequest),
}
```

#### Get File request examples

```json
{
  "type":"Get",
  "file_id":"b76f9ee2-63d2-43bd-a722-466792af7035"
}
```

#### Get File response data structure

```rust
#[derive(Deserialize)]
pub struct FileInfo {
    pub user_id: String,
    pub file_name: String,
    pub sha256: String,
    pub file_size: u32,
    pub access_path: String,
    pub task_id: Option<String>,
    pub collaborator_list: Vec<String>,
    pub allow_policy: u32,
    pub key_id: String,
}
#[derive(Deserialize)]
pub struct GetFileResponse {
    pub file_info: FileInfo,
}
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum DFSResponse {
    Get(GetFileResponse),
}
```

#### response example

```json
{
  "type":"Get",
  "file_info":{
    "user_id":"user2",
    "file_name":"file",
    "sha256":"89103c535523c3ecf9b298de94a74283cc04ee0e4072e51d02c565afb79fabaa",
    "file_size":96,
    "access_path":"/tmp/b76f9ee2-63d2-43bd-a722-466792af7035",
    "task_id":null,
    "collaborator_list":[],
    "allow_policy":0,"
    key_id":"4c9fceee-7e35-4dbb-b9e9-c35e453e1636"
  }
}
```
