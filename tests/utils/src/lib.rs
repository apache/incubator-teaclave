#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

use std::string::String;
use std::vec::Vec;

#[macro_export]
macro_rules! should_panic {
    ($fmt:expr) => {{
        match panic::catch_unwind(|| $fmt).is_err() {
            true => {}
            false => {
                ::std::rt::begin_panic($fmt, {
                    // static requires less code at runtime, more constant data
                    static _FILE_LINE_COL: (&'static str, u32, u32) = (file!(), line!(), column!());
                    &_FILE_LINE_COL
                })
            }
        }
    }};
}

#[macro_export]
macro_rules! check_all_passed {
    (
        $($f : expr),* $(,)?
    ) => {
        {
            let mut v: Vec<bool> = Vec::new();
            $(
                v.push($f);
            )*
            v.iter().all(|&x| x)
        }
    }
}

#[macro_export]
macro_rules! run_tests {
    (
        $($f : expr),* $(,)?
    ) => {
        {
            teaclave_test_utils::test_start();
            let mut ntestcases : u64 = 0u64;
            let mut failurecases : Vec<String> = Vec::new();
            $(teaclave_test_utils::test(&mut ntestcases, &mut failurecases, $f,stringify!($f));)*
            teaclave_test_utils::test_end(ntestcases, failurecases)
        }
    }
}

pub fn test_start() {
    println!("\nstart running tests");
}

pub fn test_end(ntestcases: u64, failurecases: Vec<String>) -> bool {
    let ntotal = ntestcases as usize;
    let nsucc = ntestcases as usize - failurecases.len();

    if !failurecases.is_empty() {
        print!("\nfailures: ");
        println!(
            "    {}",
            failurecases
                .iter()
                .fold(String::new(), |s, per| s + "\n    " + per)
        );
    }

    if ntotal == nsucc {
        print!("\ntest result \x1B[1;32mok\x1B[0m. ");
    } else {
        print!("\ntest result \x1B[1;31mFAILED\x1B[0m. ");
    }

    println!(
        "{} tested, {} passed, {} failed",
        ntotal,
        nsucc,
        ntotal - nsucc
    );
    failurecases.is_empty()
}

#[allow(clippy::print_literal)]
pub fn test<F, R>(ncases: &mut u64, failurecases: &mut Vec<String>, f: F, name: &str)
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    *ncases += 1;
    let t = || {
        f();
    };
    if std::panic::catch_unwind(t).is_ok() {
        println!("{} {} ... {}!", "testing", name, "\x1B[1;32mok\x1B[0m");
    } else {
        println!("{} {} ... {}!", "testing", name, "\x1B[1;31mfailed\x1B[0m");
        failurecases.push(String::from(name));
    }
}
