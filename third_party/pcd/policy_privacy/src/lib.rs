//! The privacy module for managing and controlling the privacy schemes.

#![forbid(unsafe_code)]
#![allow(unused)]

use std::{fmt::Debug, sync::RwLock};

use dp::DpManager;
use k_anon::KAnonManager;
use policy_core::{
    error::{PolicyCarryingError, PolicyCarryingResult},
    get_lock,
    policy::DpParam,
};

pub(crate) mod dp;
pub(crate) mod k_anon;

/// This struct must be wrapped within an [`Arc`].
#[derive(Default)]
pub struct PrivacyMananger {
    /// The differential privacy manager.
    dp_manager: RwLock<Option<DpManager>>,
    /// The K-anonymity manager.
    k_anon_manager: RwLock<Option<KAnonManager>>,
}

impl Debug for PrivacyMananger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "privacy manager")
    }
}

impl PrivacyMananger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the remaining privacy budget.
    pub fn dp_budget(&self) -> PolicyCarryingResult<f64> {
        let dp_manager = get_lock!(self.dp_manager, read);

        match dp_manager.as_ref() {
            Some(manager) => Ok(manager.dp_budget().0),
            None => Err(PolicyCarryingError::OperationNotSupported(
                "differential privacy not enabled".into(),
            )),
        }
    }

    /// Returns the `k`.
    pub fn k(&self) -> PolicyCarryingResult<usize> {
        let k_anon_manager = get_lock!(self.k_anon_manager, read);

        match k_anon_manager.as_ref() {
            Some(manager) => Ok(manager.k()),
            None => Err(PolicyCarryingError::OperationNotSupported(
                "k-anonymity not enabled".into(),
            )),
        }
    }

    pub fn set_dp_manager(&self, id: usize, dp_param: DpParam) -> PolicyCarryingResult<()> {
        let mut dp_manager = get_lock!(self.dp_manager, write);

        match dp_manager.as_ref() {
            Some(_) => Err(PolicyCarryingError::AlreadyLoaded),
            None => {
                dp_manager.replace(DpManager::new(id, dp_param));
                Ok(())
            }
        }
    }

    pub fn set_k_anon_manager(&self, k: usize) -> PolicyCarryingResult<()> {
        let mut k_anon_manager = get_lock!(self.k_anon_manager, write);

        match k_anon_manager.as_ref() {
            Some(_) => Err(PolicyCarryingError::AlreadyLoaded),
            None => {
                k_anon_manager.replace(KAnonManager::new(k));
                Ok(())
            }
        }
    }
}
