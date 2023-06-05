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

use teaclave_proto::teaclave_storage_service::TeaclaveStorageClient;
use teaclave_rpc::transport::Channel;
use teaclave_types::{Entry, EntryBuilder};

use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use tantivy::{
    collector::TopDocs, query::QueryParser, schema::*, DateTime, Index, IndexReader, IndexSettings,
    IndexSortByField, IndexWriter, Order, ReloadPolicy,
};

#[derive(Clone)]
pub struct Auditor {
    index: Arc<Mutex<Index>>,
    reader: Arc<Mutex<IndexReader>>,
    writer: Arc<Mutex<IndexWriter>>,
}

impl Auditor {
    pub fn try_new(
        storage: Arc<tokio::sync::Mutex<TeaclaveStorageClient<Channel>>>,
    ) -> Result<Self> {
        let directory = db_directory::DbDirectory::new(storage);

        let schema = Self::log_schema();

        let settings = IndexSettings {
            sort_by_field: Some(IndexSortByField {
                field: "date".to_string(),
                order: Order::Desc,
            }),
            ..Default::default()
        };

        let index = Index::builder()
            .schema(schema)
            .settings(settings)
            .open_or_create(directory)?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        // 8 is the max thread number of tantivy writer
        let writer = index.writer(8 * 3_000_000)?;

        let index = Arc::new(Mutex::new(index));
        let reader = Arc::new(Mutex::new(reader));
        let writer = Arc::new(Mutex::new(writer));

        Ok(Self {
            index,
            reader,
            writer,
        })
    }

    pub fn add_logs(&self, logs: Vec<Entry>) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();

        for log in logs {
            let document = Self::convert_to_doc(log);
            writer.add_document(document)?;
        }

        writer.commit()?;

        Ok(())
    }

    /// query: the query for tantivy
    /// limit: maximum number of the returned logs
    pub fn query_logs(&self, query: &str, limit: usize) -> Result<Vec<Entry>> {
        let reader = self.reader.lock().unwrap();
        let searcher = reader.searcher();
        drop(reader);

        let index = self.index.lock().unwrap();
        let schema = Self::log_schema();

        let message = schema.get_field("message").unwrap();
        let date = schema.get_field("date").unwrap();

        let query_parser = QueryParser::for_index(&index, vec![message]);
        let query = query_parser.parse_query(query)?;

        let top_docs = searcher.search(
            &query,
            &TopDocs::with_limit(limit).order_by_fast_field::<DateTime>(date),
        )?;

        let mut entries = Vec::new();

        for (_, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            let entry = Self::try_convert_to_entry(retrieved_doc)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    pub(crate) fn try_convert_to_entry(doc: Document) -> Result<Entry> {
        let schema = Self::log_schema();
        let date = schema.get_field("date").unwrap();
        let ip = schema.get_field("ip").unwrap();
        let user = schema.get_field("user").unwrap();
        let message = schema.get_field("message").unwrap();
        let result = schema.get_field("result").unwrap();

        let date = doc
            .get_first(date)
            .and_then(|d| d.as_date())
            .ok_or_else(|| anyhow!("failed to get date"))?;
        let ip = doc
            .get_first(ip)
            .and_then(|i| i.as_ip_addr())
            .ok_or_else(|| anyhow!("failed to get ip"))?;
        let user = doc
            .get_first(user)
            .and_then(|u| u.as_text())
            .ok_or_else(|| anyhow!("failed to get user"))?;
        let message = doc
            .get_first(message)
            .and_then(|m| m.as_text())
            .ok_or_else(|| anyhow!("failed to get message"))?;
        let result = doc
            .get_first(result)
            .and_then(|r| r.as_bool())
            .ok_or_else(|| anyhow!("failed to get result"))?;

        let microsecond = date.into_timestamp_micros();

        let entry = EntryBuilder::new()
            .microsecond(microsecond)
            .ip(ip)
            .user(user.to_owned())
            .message(message.to_owned())
            .result(result)
            .build();

        Ok(entry)
    }

    pub(crate) fn convert_to_doc(entry: Entry) -> Document {
        let schema = Self::log_schema();
        let date = schema.get_field("date").unwrap();
        let ip = schema.get_field("ip").unwrap();
        let user = schema.get_field("user").unwrap();
        let message = schema.get_field("message").unwrap();
        let result = schema.get_field("result").unwrap();

        let date_v = DateTime::from_timestamp_micros(entry.datetime().timestamp_micros());

        let mut doc = Document::default();
        doc.add_date(date, date_v);
        doc.add_ip_addr(ip, entry.ip());
        doc.add_text(user, &entry.user());
        doc.add_text(message, &entry.message());
        doc.add_bool(result, entry.result());

        doc
    }

    pub(crate) fn log_schema() -> Schema {
        let mut builder = Schema::builder();
        builder.add_date_field("date", INDEXED | FAST | STORED);
        builder.add_ip_addr_field("ip", INDEXED | STORED);
        builder.add_text_field("user", TEXT | STORED);
        builder.add_text_field("message", TEXT | STORED);
        builder.add_bool_field("result", INDEXED | STORED);

        builder.build()
    }
}
