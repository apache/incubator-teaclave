// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use acs_proto::*;
use mesatee_core::config::{OutboundDesc, TargetDesc};
use mesatee_core::rpc::channel::SgxTrustedChannel;
use mesatee_core::{Error, ErrorKind, Result};

use std::collections::HashSet;

pub struct ACSClient {
    channel: SgxTrustedChannel<ACSRequest, ACSResponse>,
}

impl ACSClient {
    pub fn new(target: TargetDesc) -> Result<Self> {
        let addr = target.addr;
        let channel = match target.desc {
            OutboundDesc::Sgx(enclave_addr) => {
                SgxTrustedChannel::<ACSRequest, ACSResponse>::new(addr, enclave_addr)?
            }
        };
        Ok(ACSClient { channel })
    }

    pub fn enforce_task_launch(
        &mut self,
        task: String,
        participants: HashSet<String>,
    ) -> Result<bool> {
        let req = ACSRequest::Enforce(EnforceRequest::LaunchTask(task, participants));
        let resp = self.channel.invoke(req)?;
        match resp {
            ACSResponse::Enforce(allow) => Ok(allow),
            _ => Err(Error::from(ErrorKind::RPCResponseError)),
        }
    }

    pub fn enforce_data_access(&mut self, task: String, data: String) -> Result<bool> {
        let req = ACSRequest::Enforce(EnforceRequest::AccessData(task, data));
        let resp = self.channel.invoke(req)?;
        match resp {
            ACSResponse::Enforce(allow) => Ok(allow),
            _ => Err(Error::from(ErrorKind::RPCResponseError)),
        }
    }

    pub fn enforce_data_deletion(&mut self, usr: String, data: String) -> Result<bool> {
        let req = ACSRequest::Enforce(EnforceRequest::DeleteData(usr, data));
        let resp = self.channel.invoke(req)?;
        match resp {
            ACSResponse::Enforce(allow) => Ok(allow),
            _ => Err(Error::from(ErrorKind::RPCResponseError)),
        }
    }

    pub fn enforce_script_access(&mut self, task: String, script: String) -> Result<bool> {
        let req = ACSRequest::Enforce(EnforceRequest::AccessScript(task, script));
        let resp = self.channel.invoke(req)?;
        match resp {
            ACSResponse::Enforce(allow) => Ok(allow),
            _ => Err(Error::from(ErrorKind::RPCResponseError)),
        }
    }

    pub fn enforce_script_deletion(&mut self, usr: String, script: String) -> Result<bool> {
        let req = ACSRequest::Enforce(EnforceRequest::DeleteScript(usr, script));
        let resp = self.channel.invoke(req)?;
        match resp {
            ACSResponse::Enforce(allow) => Ok(allow),
            _ => Err(Error::from(ErrorKind::RPCResponseError)),
        }
    }

    fn _announce_terms(&mut self, facts: Vec<AccessControlTerms>) -> Result<()> {
        let req = ACSRequest::Announce(AnnounceRequest { facts });
        let resp = self.channel.invoke(req)?;
        match resp {
            ACSResponse::Announce => Ok(()),
            _ => Err(Error::from(ErrorKind::RPCResponseError)),
        }
    }

    pub fn announce_task_creation(
        &mut self,
        task: String,
        creator: String,
        participants: &HashSet<String>,
    ) -> Result<()> {
        let mut facts = Vec::with_capacity(1 + participants.len());
        for par in participants {
            facts.push(AccessControlTerms::TaskParticipant(
                task.clone(),
                par.clone(),
            ));
        }
        facts.push(AccessControlTerms::TaskCreator(task, creator));
        self._announce_terms(facts)
    }

    pub fn announce_data_creation(&mut self, data: String, creator: String) -> Result<()> {
        self._announce_terms(std::vec!(AccessControlTerms::DataOwner(data, creator)))
    }

    pub fn announce_script_creation(
        &mut self,
        script: String,
        creator: String,
        is_public: bool,
    ) -> Result<()> {
        let mut terms = Vec::new();
        if is_public {
            terms.push(AccessControlTerms::IsPublicScript(script.clone()))
        }
        terms.push(AccessControlTerms::ScriptOwner(script, creator));
        self._announce_terms(terms)
    }
}
