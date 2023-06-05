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

use super::*;

use teaclave_types::EntryBuilder;

pub fn test_entry_doc_conversion() {
    let schema = Auditor::log_schema();
    let entry = EntryBuilder::new().microsecond(0).build();

    let doc = schema
        .parse_document(
            r#"{
            "date": "1970-01-01T00:00:00.00Z",
            "ip": "0000:0000:0000:0000:0000:0000:0000:0000",
            "user": "",
            "message": "",
            "result": false
        }"#,
        )
        .unwrap();

    assert_eq!(entry, Auditor::try_convert_to_entry(doc.clone()).unwrap());
    assert_eq!(Auditor::convert_to_doc(entry), doc);
}
