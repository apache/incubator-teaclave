#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::time;
#[cfg(feature = "mesalock_sgx")]
use std::untrusted::time::SystemTimeEx;

pub fn micros() -> u64 {
    loop {
        let now = time::SystemTime::now().duration_since(time::UNIX_EPOCH);

        match now {
            Err(_) => continue,
            Ok(dur) => return dur.as_secs() * 1000000 + (dur.subsec_nanos() / 1000) as u64,
        }
    }
}
