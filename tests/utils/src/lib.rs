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

pub use futures::FutureExt;

use futures::future::BoxFuture;
use std::string::String;
use std::vec::Vec;
pub use teaclave_test_utils_proc_macro::{async_test_case, test_case};
pub struct TestCase(pub String, pub fn() -> ());
pub struct AsyncTestCase(pub String, pub fn() -> BoxFuture<'static, ()>);

inventory::collect!(TestCase);
inventory::collect!(AsyncTestCase);

use std::time::Instant;
#[cfg(feature = "mesalock_sgx")]
#[allow(unused_imports)]
use std::untrusted::time::InstantEx;
#[macro_export]
macro_rules! run_inventory_tests {
    ($predicate:expr) => {{
        teaclave_test_utils::test_start();
        let mut ntestcases: u64 = 0u64;
        let mut failurecases: Vec<String> = Vec::new();

        for t in inventory::iter::<teaclave_test_utils::TestCase>.into_iter() {
            if $predicate(&t.0) {
                teaclave_test_utils::test(&mut ntestcases, &mut failurecases, t.1, &t.0);
            }
        }

        for t in inventory::iter::<teaclave_test_utils::AsyncTestCase>.into_iter() {
            if $predicate(&t.0) {
                teaclave_test_utils::async_test(&mut ntestcases, &mut failurecases, t.1, &t.0);
            }
        }
        teaclave_test_utils::test_end(ntestcases, failurecases)
    }};
    () => {
        run_inventory_tests!(|_| true);
    };
}

#[macro_export]
macro_rules! should_panic {
    ($fmt:expr) => {{
        match ::std::panic::catch_unwind(|| $fmt).is_err() {
            true => {
                println!(
                    "{} {} ... {}!",
                    "testing_should_panic",
                    stringify!($fmt),
                    "\x1B[1;32mok\x1B[0m"
                );
            }
            false => ::std::rt::begin_panic($fmt),
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

#[macro_export]
macro_rules! run_async_tests {
    (
        $($f : expr),* $(,)?
    ) => {
        {
            teaclave_test_utils::test_start();
            let mut ntestcases : u64 = 0u64;
            let mut failurecases : Vec<String> = Vec::new();
            use $crate::FutureExt;
            $(teaclave_test_utils::async_test(&mut ntestcases, &mut failurecases, || async {$f().await}.boxed(),stringify!($f));)*
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
fn do_test<F>(f: F, ncases: &mut u64, failurecases: &mut Vec<String>, name: &str)
where
    F: FnOnce() -> f64 + std::panic::UnwindSafe,
{
    *ncases += 1;
    match std::panic::catch_unwind(f) {
        Ok(elapsed) => {
            println!("{} {} ... {}!", "testing", name, "\x1B[1;32mok\x1B[0m");
            if elapsed < 0.5 {
                println!("  Elapsed time: {:?}", elapsed);
            } else {
                println!("  Elapsed time: \x1B[1;31m{:?}\x1B[0m", elapsed);
            }
        }
        Err(_) => {
            println!("{} {} ... {}!", "testing", name, "\x1B[1;31mfailed\x1B[0m");
            failurecases.push(String::from(name));
        }
    }
}

pub fn test<F, R>(ncases: &mut u64, failurecases: &mut Vec<String>, f: F, name: &str)
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    let t = || -> f64 {
        let before = Instant::now();
        f();
        before.elapsed().as_secs_f64()
    };
    do_test(t, ncases, failurecases, name)
}

pub fn async_test<F>(ncases: &mut u64, failurecases: &mut Vec<String>, f: F, name: &str)
where
    F: FnOnce() -> BoxFuture<'static, ()> + std::panic::UnwindSafe,
{
    let t = || -> f64 {
        let before = Instant::now();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(f());
        before.elapsed().as_secs_f64()
    };
    do_test(t, ncases, failurecases, name)
}
