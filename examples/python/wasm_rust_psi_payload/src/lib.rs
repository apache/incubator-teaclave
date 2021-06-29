/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::collections::HashSet;
use std::ffi::CStr;
use std::io::{BufRead, BufReader, Write};
use std::os::raw::{c_char, c_int};
use teaclave_context::TeaclaveContextFile;

#[no_mangle]
pub extern "C" fn entrypoint(argc: c_int, argv: *const *const c_char) -> i32 {
    assert_eq!(argc, 8);

    // convert `argv` to `Vec<str>`
    let argv: Vec<_> = (0..argc)
        .map(|i| unsafe { CStr::from_ptr(*argv.add(i as usize)).to_string_lossy() })
        .collect();

    // Arguments are referenced in ODD indices
    return run(
        argv[1].as_ref(),
        argv[3].as_ref(),
        argv[5].as_ref(),
        argv[7].as_ref(),
    )
    .unwrap();
}

/// This function take two input Teaclave file IDs and two output Teaclave file IDs,
/// and write identical result, the set intersection of two input files.
/// Each line of the input file will be regarded as a data entry.
/// There should be NO duplicate entries in each input file
fn run(
    input_id1: &str,
    input_id2: &str,
    output_id1: &str,
    output_id2: &str,
) -> Result<i32, std::io::Error> {
    // Create TeaclaveContextFile from file IDs
    let input1 = TeaclaveContextFile::open_input(input_id1)?;
    let input2 = TeaclaveContextFile::open_input(input_id2)?;
    let mut output1 = TeaclaveContextFile::create_output(output_id1)?;
    let mut output2 = TeaclaveContextFile::create_output(output_id2)?;

    // use HashSet to store it
    let mut output_set: HashSet<String> = HashSet::new();
    let reader1 = BufReader::new(input1);
    let mut input_set1: HashSet<String> = HashSet::new();

    // read from the first input file and save each data entry to the HashSet
    reader1
        .lines()
        .map(|l| input_set1.insert(l.unwrap()))
        .for_each(drop);

    // read from the second input file and insert to output set if the data entry
    // is also found in the first input file
    let reader2 = BufReader::new(input2);
    for line in reader2.lines() {
        if let Ok(s) = line {
            if input_set1.contains(&s) {
                output_set.insert(s);
            }
        }
    }

    let intersection_size = output_set.len();

    // write to both output files line by line
    output_set
        .iter()
        .map(|l| {
            write!(output1, "{}\n", l).unwrap();
            write!(output2, "{}\n", l).unwrap();
        })
        .for_each(drop);

    write!(output1, "done\n").unwrap();
    write!(output2, "done\n").unwrap();

    return Ok(intersection_size as _);
}
