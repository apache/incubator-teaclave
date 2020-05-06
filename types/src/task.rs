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

use crate::FunctionArguments;
use crate::Storable;
use crate::*;
use anyhow::{anyhow, bail, ensure, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Iter;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Deserialize, PartialEq, Eq, Hash, Serialize)]
pub struct UserID(String);

impl std::convert::From<String> for UserID {
    fn from(uid: String) -> UserID {
        UserID(uid)
    }
}

impl std::convert::From<&str> for UserID {
    fn from(uid: &str) -> UserID {
        UserID(uid.to_string())
    }
}

impl std::convert::From<UserID> for String {
    fn from(user_id: UserID) -> String {
        user_id.to_string()
    }
}

impl std::fmt::Display for UserID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type UserList = OwnerList;

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct OwnerList {
    pub uids: HashSet<UserID>,
}

impl OwnerList {
    pub fn new<T: IntoIterator>(uids: T) -> Self
    where
        <T as IntoIterator>::Item: ToString,
    {
        OwnerList {
            uids: uids.into_iter().map(|x| UserID(x.to_string())).collect(),
        }
    }

    pub fn contains(&self, uid: &UserID) -> bool {
        self.uids.contains(uid)
    }

    pub fn len(&self) -> usize {
        self.uids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, value: UserID) -> bool {
        self.uids.insert(value)
    }

    pub fn union(mut self, other: Self) -> Self {
        for value in other.uids {
            self.uids.insert(value);
        }
        self
    }

    pub fn unions<I>(i: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        i.into_iter().fold(Self::default(), Self::union)
    }
}

impl<T> std::convert::From<Vec<T>> for OwnerList
where
    T: ToString,
{
    fn from(owners: Vec<T>) -> Self {
        OwnerList::new(owners)
    }
}

impl std::convert::From<OwnerList> for Vec<String> {
    fn from(owners: OwnerList) -> Vec<String> {
        owners.into_iter().map(|uid| uid.to_string()).collect()
    }
}

impl IntoIterator for OwnerList {
    type Item = UserID;
    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.uids.into_iter()
    }
}

#[derive(Debug, Deserialize, Serialize, std::cmp::PartialEq)]
pub enum TaskStatus {
    Created,
    DataAssigned,
    Approved,
    Staged,
    Running,
    Finished,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Created
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OutputsTags {
    inner: HashMap<String, FileAuthTag>,
}

impl OutputsTags {
    pub fn new(hm: HashMap<String, FileAuthTag>) -> Self {
        Self { inner: hm }
    }

    pub fn iter(&self) -> Iter<String, FileAuthTag> {
        self.inner.iter()
    }

    pub fn get(&self, key: &str) -> Option<&FileAuthTag> {
        self.inner.get(key)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::convert::TryFrom<HashMap<String, String>> for OutputsTags {
    type Error = anyhow::Error;
    fn try_from(input: HashMap<String, String>) -> Result<Self> {
        let mut ret = HashMap::with_capacity(input.len());
        for (k, v) in input.iter() {
            let tag = FileAuthTag::from_hex(v)?;
            ret.insert(k.to_string(), tag);
        }
        Ok(OutputsTags::new(ret))
    }
}

impl<S: std::default::Default + std::hash::BuildHasher> std::convert::From<OutputsTags>
    for HashMap<String, String, S>
{
    fn from(tags: OutputsTags) -> HashMap<String, String, S> {
        tags.iter()
            .map(|(k, v)| (k.to_string(), v.to_hex()))
            .collect()
    }
}

impl std::iter::FromIterator<(String, FileAuthTag)> for OutputsTags {
    fn from_iter<T: IntoIterator<Item = (String, FileAuthTag)>>(iter: T) -> Self {
        OutputsTags {
            inner: HashMap::from_iter(iter),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskOutputs {
    pub return_value: Vec<u8>,
    pub tags_map: OutputsTags,
}

impl TaskOutputs {
    pub fn new(value: impl Into<Vec<u8>>, tags_map: HashMap<String, FileAuthTag>) -> Self {
        TaskOutputs {
            return_value: value.into(),
            tags_map: OutputsTags::new(tags_map),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskFailure {
    pub reason: String,
}

impl TaskFailure {
    pub fn new(reason: impl ToString) -> Self {
        TaskFailure {
            reason: reason.to_string(),
        }
    }
}

impl std::fmt::Display for TaskFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TaskFailure {}", self.reason)
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct ExternalID {
    pub prefix: String,
    pub uuid: Uuid,
}

impl ExternalID {
    pub fn new(prefix: impl ToString, uuid: Uuid) -> Self {
        ExternalID {
            prefix: prefix.to_string(),
            uuid,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl ToString for ExternalID {
    fn to_string(&self) -> String {
        format!("{}-{}", self.prefix, self.uuid.to_string())
    }
}

impl std::convert::From<&ExternalID> for ExternalID {
    fn from(s: &ExternalID) -> ExternalID {
        s.clone()
    }
}

impl std::convert::TryFrom<&str> for ExternalID {
    type Error = anyhow::Error;
    fn try_from(ext_id: &str) -> Result<Self> {
        let pos = ext_id
            .find('-')
            .ok_or_else(|| anyhow!("Invalid external id: {}", ext_id))?;
        let (part0, part1) = ext_id.split_at(pos);
        ensure!(part1.len() > 1, "Invalid external id: {}", ext_id);
        let eid = ExternalID {
            prefix: part0.to_string(),
            uuid: Uuid::parse_str(&part1[1..])?,
        };
        Ok(eid)
    }
}

impl std::convert::TryFrom<String> for ExternalID {
    type Error = anyhow::Error;
    fn try_from(ext_id: String) -> Result<Self> {
        ext_id.as_str().try_into()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TaskResult {
    NotReady,
    Ok(TaskOutputs),
    Err(TaskFailure),
}

impl TaskResult {
    pub fn is_ok(&self) -> bool {
        match self {
            TaskResult::Ok(_) => true,
            _ => false,
        }
    }

    #[cfg(test_mode)]
    pub fn unwrap(self) -> TaskOutputs {
        match self {
            TaskResult::Ok(t) => t,
            TaskResult::Err(e) => {
                panic!("called `TaskResult::unwrap()` on an `Err` value: {:?}", &e)
            }
            TaskResult::NotReady => panic!("called `TaskResult::unwrap()` on NotReady case"),
        }
    }
}

impl Default for TaskResult {
    fn default() -> Self {
        TaskResult::NotReady
    }
}

// This is intended for proto::TaskResult field
// Since proto::TaskResult is a wrapper of One-Of keywords,
// it is always converted to an Option<proto::TaskResult>
// when referenced in a request/response structure.
impl<T> std::convert::TryFrom<Option<T>> for TaskResult
where
    T: TryInto<TaskResult, Error = Error>,
{
    type Error = Error;
    fn try_from(option: Option<T>) -> Result<Self> {
        let ret = match option {
            Some(result) => result.try_into()?,
            None => unreachable!(),
        };
        Ok(ret)
    }
}

impl<T, E> std::convert::From<TaskResult> for Option<std::result::Result<T, E>>
where
    T: From<TaskOutputs>,
    E: From<TaskFailure>,
{
    fn from(task_result: TaskResult) -> Option<std::result::Result<T, E>> {
        match task_result {
            TaskResult::Ok(t) => Some(Ok(t.into())),
            TaskResult::Err(e) => Some(Err(e.into())),
            TaskResult::NotReady => None,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TaskFileOwners {
    inner: HashMap<String, OwnerList>,
}

impl TaskFileOwners {
    pub fn all_owners(&self) -> OwnerList {
        OwnerList::unions(self.inner.values().cloned())
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<String, OwnerList> {
        self.inner.keys()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn get(&self, key: &str) -> Option<&OwnerList> {
        self.inner.get(key)
    }

    pub fn check(&self, fkey: &str, fowners: &OwnerList) -> Result<()> {
        match self.inner.get(fkey) {
            Some(owner_list) => {
                ensure!(
                    owner_list == fowners,
                    "Assign: file ownership mismatch. {:?}",
                    fkey
                );
            }
            None => bail!("Assign: file name not exist in ownership spec. {:?}", fkey),
        };
        Ok(())
    }
}

impl<V> std::iter::FromIterator<(String, V)> for TaskFileOwners
where
    V: Into<OwnerList>,
{
    fn from_iter<T: IntoIterator<Item = (String, V)>>(iter: T) -> Self {
        TaskFileOwners {
            inner: HashMap::from_iter(iter.into_iter().map(|(k, v)| (k, v.into()))),
        }
    }
}

impl IntoIterator for TaskFileOwners {
    type Item = (String, OwnerList);
    type IntoIter = std::collections::hash_map::IntoIter<String, OwnerList>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl std::convert::From<HashMap<String, OwnerList>> for TaskFileOwners {
    fn from(input: HashMap<String, OwnerList>) -> Self {
        input.into_iter().collect()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskFiles<T: Clone> {
    inner: HashMap<String, T>,
}

impl<T> Default for TaskFiles<T>
where
    T: Clone,
{
    fn default() -> TaskFiles<T> {
        TaskFiles::<T> {
            inner: HashMap::new(),
        }
    }
}

impl<T> TaskFiles<T>
where
    T: Clone + Storable,
{
    pub fn assign(&mut self, fname: &str, file: T) -> Result<()> {
        ensure!(
            self.inner.get(fname).is_none(),
            "Assign: file already assigned. {:?}",
            fname
        );
        self.inner.insert(fname.to_owned(), file);
        Ok(())
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<String, T> {
        self.inner.keys()
    }

    pub fn external_ids(&self) -> HashMap<String, ExternalID> {
        self.inner
            .iter()
            .map(|(fname, file)| (fname.to_string(), file.external_id()))
            .collect()
    }
}

impl TaskFiles<TeaclaveOutputFile> {
    pub fn update_cmac(
        &mut self,
        fname: &str,
        auth_tag: &FileAuthTag,
    ) -> Result<&TeaclaveOutputFile> {
        let file = match self.inner.get_mut(fname) {
            Some(file) => {
                file.assign_cmac(auth_tag)?;
                file
            }
            _ => bail!("Upadate_cmac: file not found. {:?}", fname),
        };

        Ok(file)
    }
}

impl<T> IntoIterator for TaskFiles<T>
where
    T: Clone,
{
    type Item = (String, T);
    type IntoIter = std::collections::hash_map::IntoIter<String, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl std::convert::From<TaskFiles<TeaclaveInputFile>> for FunctionInputFiles {
    fn from(files: TaskFiles<TeaclaveInputFile>) -> Self {
        files.into_iter().collect()
    }
}

impl std::convert::From<TaskFiles<TeaclaveOutputFile>> for FunctionOutputFiles {
    fn from(files: TaskFiles<TeaclaveOutputFile>) -> Self {
        files.into_iter().collect()
    }
}

const TASK_PREFIX: &str = "task";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Task {
    pub task_id: Uuid,
    pub creator: UserID,
    pub function_id: ExternalID,
    pub function_arguments: FunctionArguments,
    pub executor: Executor,
    pub inputs_ownership: TaskFileOwners,
    pub outputs_ownership: TaskFileOwners,
    pub function_owner: UserID,
    pub participants: UserList,
    pub approved_users: UserList,
    pub assigned_inputs: TaskFiles<TeaclaveInputFile>,
    pub assigned_outputs: TaskFiles<TeaclaveOutputFile>,
    pub result: TaskResult,
    pub status: TaskStatus,
}

impl Storable for Task {
    fn key_prefix() -> &'static str {
        TASK_PREFIX
    }

    fn uuid(&self) -> Uuid {
        self.task_id
    }
}

impl Task {
    pub fn new(
        requester: UserID,
        req_executor: Executor,
        req_func_args: FunctionArguments,
        req_input_owners: impl Into<TaskFileOwners>,
        req_output_owners: impl Into<TaskFileOwners>,
        function: Function,
    ) -> Result<Self> {
        let req_input_owners = req_input_owners.into();
        let req_output_owners = req_output_owners.into();

        // gather all participants
        let input_owners = req_input_owners.all_owners();
        let output_owners = req_output_owners.all_owners();
        let mut participants = UserList::unions(vec![input_owners, output_owners]);
        participants.insert(requester.clone());
        if !function.public {
            participants.insert(function.owner.clone());
        }

        //check function compatibility
        let fn_args_spec: HashSet<&String> = function.arguments.iter().collect();
        let req_args: HashSet<&String> = req_func_args.inner().keys().collect();
        ensure!(fn_args_spec == req_args, "function_arguments mismatch");

        // check input fkeys
        let inputs_spec: HashSet<&String> = function.inputs.iter().map(|f| &f.name).collect();
        let req_input_fkeys: HashSet<&String> = req_input_owners.keys().collect();
        ensure!(inputs_spec == req_input_fkeys, "input keys mismatch");

        // check output fkeys
        let outputs_spec: HashSet<&String> = function.outputs.iter().map(|f| &f.name).collect();
        let req_output_fkeys: HashSet<&String> = req_output_owners.keys().collect();
        ensure!(outputs_spec == req_output_fkeys, "output keys mismatch");

        // Skip the assignment if no file is required
        let status = if req_input_owners.is_empty() && req_output_owners.is_empty() {
            TaskStatus::DataAssigned
        } else {
            TaskStatus::Created
        };

        let task = Task {
            task_id: Uuid::new_v4(),
            creator: requester,
            executor: req_executor,
            function_id: function.external_id(),
            function_owner: function.owner.clone(),
            function_arguments: req_func_args,
            inputs_ownership: req_input_owners,
            outputs_ownership: req_output_owners,
            participants,
            status,
            ..Default::default()
        };

        Ok(task)
    }

    pub fn approve(&mut self, requester: &UserID) -> Result<()> {
        ensure!(
            self.status == TaskStatus::DataAssigned,
            "Unexpected task status when approving: {:?}",
            self.status
        );

        ensure!(
            self.participants.contains(requester),
            "Unexpected user trying to approve a task: {:?}",
            requester
        );

        self.approved_users.insert(requester.clone());
        if self.participants == self.approved_users {
            self.update_status(TaskStatus::Approved);
        }

        Ok(())
    }

    pub fn invoking_by_executor(&mut self) -> Result<()> {
        ensure!(
            self.status == TaskStatus::Staged,
            "Unexpected task status when invoked: {:?}",
            self.status
        );
        self.status = TaskStatus::Running;
        Ok(())
    }

    pub fn finish(&mut self, result: TaskResult) -> Result<()> {
        ensure!(
            self.status == TaskStatus::Running,
            "Unexpected task status when invoked: {:?}",
            self.status
        );
        self.result = result;
        self.status = TaskStatus::Finished;
        Ok(())
    }

    pub fn stage_for_running(
        &mut self,
        requester: &UserID,
        function: Function,
    ) -> Result<StagedTask> {
        ensure!(
            &self.creator == requester,
            "Unexpected user trying to invoke a task: {:?}",
            requester
        );
        ensure!(
            self.status == TaskStatus::Approved,
            "Unexpected task status when invoked: {:?}",
            self.status
        );
        let function_arguments = self.function_arguments.clone();
        let staged_task = StagedTask {
            task_id: self.task_id,
            executor: self.executor,
            executor_type: function.executor_type,
            function_id: function.id,
            function_name: function.name,
            function_payload: function.payload,
            function_arguments,
            input_data: self.assigned_inputs.clone().into(),
            output_data: self.assigned_outputs.clone().into(),
        };

        self.update_status(TaskStatus::Staged);
        Ok(staged_task)
    }

    pub fn assign_input(
        &mut self,
        requester: &UserID,
        fname: &str,
        file: TeaclaveInputFile,
    ) -> Result<()> {
        ensure!(
            self.status == TaskStatus::Created,
            "Unexpected task status during input assignment: {:?}",
            self.status
        );

        ensure!(
            file.owner.contains(requester),
            "Assign: requester is not in the owner list. {:?}.",
            file.external_id()
        );

        self.inputs_ownership.check(fname, &file.owner)?;

        self.assigned_inputs.assign(fname, file)?;

        if self.all_data_assigned() {
            self.update_status(TaskStatus::DataAssigned);
        }
        Ok(())
    }

    pub fn assign_output(
        &mut self,
        requester: &UserID,
        fname: &str,
        file: TeaclaveOutputFile,
    ) -> Result<()> {
        ensure!(
            self.status == TaskStatus::Created,
            "Unexpected task status during output assignment: {:?}",
            self.status
        );

        ensure!(
            file.owner.contains(requester),
            "Assign: requester is not in the owner list. {:?}.",
            file.external_id()
        );

        self.outputs_ownership.check(fname, &file.owner)?;

        self.assigned_outputs.assign(fname, file)?;

        if self.all_data_assigned() {
            self.update_status(TaskStatus::DataAssigned);
        }
        Ok(())
    }

    fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    fn all_data_assigned(&self) -> bool {
        let input_args: HashSet<&String> = self.inputs_ownership.keys().collect();
        let assiged_inputs: HashSet<&String> = self.assigned_inputs.keys().collect();
        if input_args != assiged_inputs {
            return false;
        }

        let output_args: HashSet<&String> = self.outputs_ownership.keys().collect();
        let assiged_outputs: HashSet<&String> = self.assigned_outputs.keys().collect();
        if output_args != assiged_outputs {
            return false;
        }

        true
    }
}
