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

use std::string::String;
use std::vec::Vec;

macro_rules! unit_tests {
    (
        $($f : expr),* $(,)?
    ) => {
        {
            unit_test_start();
            let mut ntestcases : u64 = 0u64;
            let mut failurecases : Vec<String> = Vec::new();
            $(unit_test(&mut ntestcases, &mut failurecases, $f,stringify!($f));)*
            unit_test_end(ntestcases, failurecases)
        }
    }
}

pub fn unit_test_start() {
    println!("\nstart running tests");
}

pub fn unit_test_end(ntestcases: u64, failurecases: Vec<String>) -> usize {
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
    failurecases.len()
}

pub fn unit_test<F, R>(ncases: &mut u64, failurecases: &mut Vec<String>, f: F, name: &str)
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    *ncases += 1;
    let ret = std::panic::catch_unwind(|| {
        f();
    });
    if ret.is_ok() {
        println!("testing {} ... \x1B[1;32mok\x1B[0m!", name);
    } else {
        println!("testing {} ... \x1B[1;31mfailed\x1B[0m!", name);
        failurecases.push(String::from(name));
    }
}
