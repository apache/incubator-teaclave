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

// Insert std prelude in the top for the sgx feature

#[cfg(feature = "mesalock_sgx")]

use std::prelude::v1::*;

use sgx_types::SGX_HASH_SIZE;

pub fn oget_intersection(
    a: &[[u8; SGX_HASH_SIZE]],
    b: &[[u8; SGX_HASH_SIZE]],
    v1: &mut Vec<u8>,
    v2: &mut Vec<u8>,
) {
    let n = a.len();
    for i in 0..n {
        let ret = obinary_search(b, &a[i], v2);
        let miss = oequal(usize::max_value(), ret as usize);
        v1[i] = omov(miss as isize, 0, 1) as u8;
    }
}

pub fn obinary_search(
    b: &[[u8; SGX_HASH_SIZE]],
    target: &[u8; SGX_HASH_SIZE],
    v2: &mut Vec<u8>,
) -> isize {
    let mut lo: isize = 0;
    let mut hi: isize = b.len() as isize - 1;
    let mut ret: isize = -1;

    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let hit = eq(&b[mid as usize], target);
        ret = omov(hit, mid, ret);
        v2[mid as usize] = omov(hit, 1, v2[mid as usize] as isize) as u8;
        let be = le(&b[mid as usize], target);
        lo = omov(be, mid + 1, lo);
        hi = omov(be, hi, mid - 1);
    }
    ret
}

pub fn eq(a: &[u8; SGX_HASH_SIZE], b: &[u8; SGX_HASH_SIZE]) -> isize {
    let mut ret: isize = 1;
    for i in 0..SGX_HASH_SIZE {
        let hit = oequal(a[i] as usize, b[i] as usize);
        ret = omov(hit as isize, ret, 0);
    }
    ret
}

pub fn le(a: &[u8; SGX_HASH_SIZE], b: &[u8; SGX_HASH_SIZE]) -> isize {
    let mut ret: isize = 0;
    for i in 0..SGX_HASH_SIZE {
        let hit = oequal(a[i] as usize, b[i] as usize);
        let be = ob(a[i] as usize, b[i] as usize);
        let cmp = omov(hit as isize, 0, omov(be as isize, -1, 1));
        ret = omov(ret, ret, cmp)
    }
    (ret <= 0) as isize
}

pub fn oequal(x: usize, y: usize) -> bool {
    let ret: bool;
    unsafe {
        llvm_asm!(
            "cmp %rcx, %rdx \n\t
             sete %al \n\t"
            : "={al}"(ret)
            : "{rcx}"(x), "{rdx}" (y)
            : "rcx", "rdx"
            : "volatile"
        );
    }
    ret
}

pub fn ob(x: usize, y: usize) -> bool {
    let ret: bool;
    unsafe {
        llvm_asm!(
            "cmp %rdx, %rcx \n\t
             setb %al \n\t"
            : "={al}"(ret)
            : "{rcx}"(x), "{rdx}" (y)
            : "rcx", "rdx"
            : "volatile"
        );
    }
    ret
}

pub fn omov(flag: isize, x: isize, y: isize) -> isize {
    let ret: isize;
    unsafe {
        llvm_asm!(
            "xor %rcx, %rcx \n\t
             mov $1, %rcx \n\t
             test %rcx, %rcx \n\t
             cmovz %rdx, %rax \n\t"
            : "={rax}"(ret)
            : "r"(flag), "{rax}" (x), "{rdx}" (y)
            : "rax", "rcx", "rdx"
            : "volatile"
        );
    }
    ret
}
