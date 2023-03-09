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

use crate::*;
use anyhow::{bail, ensure, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::TryInto;
use uuid::Uuid;

const TASK_PREFIX: &str = "task";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TaskState {
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

impl Storable for TaskState {
    fn key_prefix() -> &'static str {
        TASK_PREFIX
    }

    fn uuid(&self) -> Uuid {
        self.task_id
    }
}

impl TaskState {
    pub fn everyone_approved(&self) -> bool {
        // Single user task is by default approved by the creator
        (self.participants.len() == 1) || (self.participants == self.approved_users)
    }

    pub fn all_data_assigned(&self) -> bool {
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

    pub fn has_participant(&self, user_id: &UserID) -> bool {
        self.participants.contains(user_id)
    }

    pub fn has_creator(&self, user_id: &UserID) -> bool {
        &self.creator == user_id
    }

    pub fn is_ended(&self) -> bool {
        matches!(
            self.status,
            TaskStatus::Finished | TaskStatus::Failed | TaskStatus::Canceled
        )
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Task<S: StateTag> {
    state: TaskState,
    extra: S,
}

pub trait StateTag {}
impl StateTag for Create {}
impl StateTag for Assign {}
impl StateTag for Approve {}
impl StateTag for Stage {}
impl StateTag for Run {}
impl StateTag for Finish {}
impl StateTag for Done {}
impl StateTag for Cancel {}
impl StateTag for Fail {}

impl Task<Create> {
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
        let fn_args_spec: HashSet<&String> = function
            .arguments
            .iter()
            .filter(|arg| arg.allow_overwrite)
            .map(|arg| &arg.key)
            .collect();
        let req_args: HashSet<&String> = req_func_args.inner().keys().collect();
        ensure!(fn_args_spec == req_args, "function_arguments mismatch");

        let mut func_args = req_func_args;
        for arg in &function.arguments {
            if !arg.allow_overwrite || !func_args.inner().contains_key(&arg.key) {
                func_args.insert(
                    arg.key.clone(),
                    serde_json::Value::String(arg.default_value.clone()),
                );
            }
        }

        // check input fkeys
        let inputs_spec: HashSet<&String> = function.inputs.iter().map(|f| &f.name).collect();
        let mut req_input_fkeys: HashSet<&String> = req_input_owners.keys().collect();
        // If an input/output file is marked with `optional: True`, users do not need to
        // register the file.
        let option_inputs_spec: HashSet<&String> = function
            .inputs
            .iter()
            .filter(|f| f.optional)
            .map(|f| &f.name)
            .collect();
        req_input_fkeys.extend(&option_inputs_spec);

        ensure!(inputs_spec == req_input_fkeys, "input keys mismatch");

        // check output fkeys
        let outputs_spec: HashSet<&String> = function.outputs.iter().map(|f| &f.name).collect();
        let mut req_output_fkeys: HashSet<&String> = req_output_owners.keys().collect();
        let option_outputs_spec: HashSet<&String> = function
            .outputs
            .iter()
            .filter(|f| f.optional)
            .map(|f| &f.name)
            .collect();
        req_output_fkeys.extend(&option_outputs_spec);
        ensure!(outputs_spec == req_output_fkeys, "output keys mismatch");

        let ts = TaskState {
            task_id: Uuid::new_v4(),
            creator: requester,
            executor: req_executor,
            function_id: function.external_id(),
            function_owner: function.owner.clone(),
            function_arguments: func_args,
            inputs_ownership: req_input_owners,
            outputs_ownership: req_output_owners,
            participants,
            ..Default::default()
        };

        Ok(Task {
            state: ts,
            extra: Create,
        })
    }
}

impl Task<Assign> {
    pub fn new(ts: TaskState) -> Result<Self> {
        let task = Task::<Assign> {
            state: ts,
            extra: Assign,
        };
        Ok(task)
    }

    pub fn assign_input(
        &mut self,
        requester: &UserID,
        fname: &str,
        file: TeaclaveInputFile,
    ) -> Result<()> {
        ensure!(
            file.owner.contains(requester),
            "Assign: requester is not in the owner list. {:?}.",
            file.external_id()
        );

        self.state.inputs_ownership.check(fname, &file.owner)?;
        self.state.assigned_inputs.assign(fname, file)?;
        Ok(())
    }

    pub fn assign_output(
        &mut self,
        requester: &UserID,
        fname: &str,
        file: TeaclaveOutputFile,
    ) -> Result<()> {
        ensure!(
            file.owner.contains(requester),
            "Assign: requester is not in the owner list. {:?}.",
            file.external_id()
        );

        self.state.outputs_ownership.check(fname, &file.owner)?;
        self.state.assigned_outputs.assign(fname, file)?;
        Ok(())
    }
}

impl Task<Approve> {
    pub fn new(ts: TaskState) -> Result<Self> {
        let task = Task::<Approve> {
            state: ts,
            extra: Approve,
        };
        Ok(task)
    }

    pub fn approve(&mut self, requester: &UserID) -> Result<()> {
        ensure!(
            self.state.participants.contains(requester),
            "Unexpected user trying to approve a task: {:?}",
            requester
        );

        self.state.approved_users.insert(requester.clone());
        Ok(())
    }
}

impl Task<Stage> {
    pub fn new(ts: TaskState) -> Result<Self> {
        let task = Task::<Stage> {
            state: ts,
            extra: Stage,
        };
        Ok(task)
    }

    pub fn stage_for_running(
        &mut self,
        requester: &UserID,
        function: Function,
    ) -> Result<StagedTask> {
        ensure!(
            self.state.has_creator(requester),
            "Requestor is not the task creater"
        );

        let function_arguments = self.state.function_arguments.clone();
        let staged_task = StagedTask {
            task_id: self.state.task_id,
            user_id: requester.into(),
            executor: self.state.executor,
            executor_type: function.executor_type,
            function_id: function.id,
            function_name: function.name,
            function_payload: function.payload,
            function_arguments,
            input_data: self.state.assigned_inputs.clone().into(),
            output_data: self.state.assigned_outputs.clone().into(),
        };
        Ok(staged_task)
    }
}

impl Task<Run> {
    pub fn new(ts: TaskState) -> Result<Self> {
        let task = Task::<Run> {
            state: ts,
            extra: Run,
        };
        Ok(task)
    }
}

impl Task<Finish> {
    pub fn new(ts: TaskState) -> Result<Self> {
        let task = Task::<Finish> {
            state: ts,
            extra: Finish,
        };
        Ok(task)
    }

    pub fn update_output_cmac(
        &mut self,
        fname: &str,
        auth_tag: &FileAuthTag,
    ) -> Result<&TeaclaveOutputFile> {
        self.state.assigned_outputs.update_cmac(fname, auth_tag)
    }

    pub fn update_result(&mut self, result: TaskResult) -> Result<()> {
        self.state.result = result;
        Ok(())
    }
}

impl Task<Done> {
    pub fn new(ts: TaskState) -> Result<Self> {
        let task = Task::<Done> {
            state: ts,
            extra: Done,
        };
        Ok(task)
    }
}

impl Task<Fail> {
    pub fn new(ts: TaskState) -> Result<Self> {
        let task = Task::<Fail> {
            state: ts,
            extra: Fail,
        };
        Ok(task)
    }

    pub fn update_result(&mut self, result: TaskResult) -> Result<()> {
        match &result {
            TaskResult::Err(_) => {
                self.state.result = result;
                Ok(())
            }
            _ => Err(Error::msg(
                "TaskResult::Err(TaskFailure) is expected for failed task",
            )),
        }
    }
}

impl Task<Cancel> {
    pub fn new(ts: TaskState) -> Result<Self> {
        let task = Task::<Cancel> {
            state: ts,
            extra: Cancel,
        };
        Ok(task)
    }

    pub fn update_result(&mut self, result: TaskResult) -> Result<()> {
        match &result {
            TaskResult::Err(_) => {
                self.state.result = result;
                Ok(())
            }
            _ => Err(Error::msg(
                "TaskResult::Err(TaskFailure) is expected for canceled task",
            )),
        }
    }
}

trait TryTransitionTo<T>: Sized {
    type Error;
    fn try_transition_to(self) -> std::result::Result<T, Error>;
    fn ready_for_transition(&self) -> bool {
        true
    }
}

impl TryTransitionTo<Task<Approve>> for Task<Assign> {
    type Error = Error;
    fn try_transition_to(self) -> Result<Task<Approve>> {
        ensure!(self.ready_for_transition(), "Not ready: Assign -> Approve");
        Task::<Approve>::new(self.state)
    }

    fn ready_for_transition(&self) -> bool {
        self.state.all_data_assigned()
    }
}

impl TryTransitionTo<Task<Stage>> for Task<Approve> {
    type Error = Error;
    fn try_transition_to(self) -> Result<Task<Stage>> {
        ensure!(self.ready_for_transition(), "Not ready: Apporve -> Stage");
        Task::<Stage>::new(self.state)
    }
    fn ready_for_transition(&self) -> bool {
        self.state.everyone_approved()
    }
}

impl TryTransitionTo<Task<Run>> for Task<Stage> {
    type Error = Error;
    fn try_transition_to(self) -> Result<Task<Run>> {
        Task::<Run>::new(self.state)
    }
}

impl TryTransitionTo<Task<Finish>> for Task<Run> {
    type Error = Error;
    fn try_transition_to(self) -> Result<Task<Finish>> {
        Task::<Finish>::new(self.state)
    }
}

impl TryTransitionTo<Task<Done>> for Task<Finish> {
    type Error = Error;
    fn try_transition_to(self) -> Result<Task<Done>> {
        Task::<Done>::new(self.state)
    }
}

impl std::convert::TryFrom<TaskState> for Task<Assign> {
    type Error = Error;

    fn try_from(ts: TaskState) -> Result<Self> {
        let task = match ts.status {
            TaskStatus::Created => Task::<Assign>::new(ts)?,
            _ => bail!("Cannot restore to Assign from saved state "),
        };
        Ok(task)
    }
}

impl std::convert::TryFrom<TaskState> for Task<Approve> {
    type Error = Error;

    fn try_from(ts: TaskState) -> Result<Self> {
        let task = match ts.status {
            TaskStatus::Created => {
                let task: Task<Assign> = ts.try_into()?;
                task.try_transition_to()?
            }
            TaskStatus::DataAssigned => Task::<Approve>::new(ts)?,
            _ => bail!("Cannot restore to Approve from saved state"),
        };
        Ok(task)
    }
}

impl std::convert::TryFrom<TaskState> for Task<Stage> {
    type Error = Error;

    fn try_from(ts: TaskState) -> Result<Self> {
        let task = match ts.status {
            TaskStatus::Created | TaskStatus::DataAssigned => {
                let task: Task<Approve> = ts.try_into()?;
                task.try_transition_to()?
            }
            TaskStatus::Approved => Task::<Stage>::new(ts)?,
            _ => bail!("Cannot restore to Stage from saved state"),
        };
        Ok(task)
    }
}

impl std::convert::TryFrom<TaskState> for Task<Run> {
    type Error = Error;

    fn try_from(ts: TaskState) -> Result<Self> {
        let task = match ts.status {
            TaskStatus::Staged => Task::<Run>::new(ts)?,
            _ => bail!("Cannot restore to Run from saved state"),
        };
        Ok(task)
    }
}

impl std::convert::TryFrom<TaskState> for Task<Finish> {
    type Error = Error;

    fn try_from(ts: TaskState) -> Result<Self> {
        let task = match ts.status {
            TaskStatus::Running => Task::<Finish>::new(ts)?,
            _ => bail!("Cannot restore to Finish from saved state"),
        };
        Ok(task)
    }
}

impl std::convert::TryFrom<TaskState> for Task<Fail> {
    type Error = Error;

    fn try_from(ts: TaskState) -> Result<Self> {
        let task = match ts.status {
            TaskStatus::Running | TaskStatus::Staged => Task::<Fail>::new(ts)?,
            _ => bail!("Cannot restore to Fail from saved state"),
        };
        Ok(task)
    }
}

impl std::convert::TryFrom<TaskState> for Task<Cancel> {
    type Error = Error;

    fn try_from(ts: TaskState) -> Result<Self> {
        let task = match ts.status {
            TaskStatus::Running
            | TaskStatus::Staged
            | TaskStatus::Approved
            | TaskStatus::Created
            | TaskStatus::DataAssigned => Task::<Cancel>::new(ts)?,
            _ => bail!("Cannot restore to Cancel from saved state"),
        };
        Ok(task)
    }
}

impl std::convert::From<Task<Create>> for TaskState {
    fn from(mut task: Task<Create>) -> TaskState {
        task.state.status = TaskStatus::Created;
        task.state
    }
}

impl std::convert::From<Task<Fail>> for TaskState {
    fn from(mut task: Task<Fail>) -> TaskState {
        task.state.status = TaskStatus::Failed;
        task.state
    }
}

impl std::convert::From<Task<Cancel>> for TaskState {
    fn from(mut task: Task<Cancel>) -> TaskState {
        task.state.status = TaskStatus::Canceled;
        task.state
    }
}

impl_transit_and_into_task_state!(Assign => Approve);
impl_transit_and_into_task_state!(Approve => Stage);
impl_transit_and_into_task_state!(Stage => Run);
impl_transit_and_into_task_state!(Run => Finish);
impl_transit_and_into_task_state!(Finish => Done);

#[macro_export]
macro_rules! impl_transit_and_into_task_state {
    ( $cur:ty => $next:ty ) => {
        impl std::convert::From<Task<$cur>> for TaskState {
            fn from(mut task: Task<$cur>) -> TaskState {
                if <Task<$cur> as TryTransitionTo<Task<$next>>>::ready_for_transition(&task) {
                    // We assume that if it's ready for transistion, the result is always Ok.
                    let mut nt: Task<$next> = task.try_transition_to().unwrap();
                    nt.state.status = nt.extra.into();
                    return nt.state;
                }
                task.state.status = task.extra.into();
                task.state
            }
        }
    };
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Create;
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Assign;
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Approve;
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Stage;
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Run;
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Finish;
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Done;
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Cancel;
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Fail;

impl std::convert::From<Create> for TaskStatus {
    fn from(_tag: Create) -> TaskStatus {
        TaskStatus::Created
    }
}

impl std::convert::From<Assign> for TaskStatus {
    fn from(_tag: Assign) -> TaskStatus {
        TaskStatus::Created
    }
}

impl std::convert::From<Approve> for TaskStatus {
    fn from(_tag: Approve) -> TaskStatus {
        TaskStatus::DataAssigned
    }
}

impl std::convert::From<Stage> for TaskStatus {
    fn from(_tag: Stage) -> TaskStatus {
        TaskStatus::Approved
    }
}

impl std::convert::From<Run> for TaskStatus {
    fn from(_tag: Run) -> TaskStatus {
        TaskStatus::Staged
    }
}

impl std::convert::From<Finish> for TaskStatus {
    fn from(_tag: Finish) -> TaskStatus {
        TaskStatus::Running
    }
}

impl std::convert::From<Done> for TaskStatus {
    fn from(_tag: Done) -> TaskStatus {
        TaskStatus::Finished
    }
}
