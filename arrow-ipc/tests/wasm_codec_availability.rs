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

#[cfg(feature = "zstd")]
use std::io::Cursor;

#[cfg(feature = "zstd")]
use arrow_ipc::reader::StreamReader;

#[cfg(any(
    not(feature = "zstd"),
    all(feature = "zstd", target_arch = "wasm32", target_os = "unknown")
))]
mod writer_failure {
    use std::cell::RefCell;
    use std::io::Write;
    use std::rc::Rc;
    use std::sync::Arc;

    use arrow_array::{Int32Array, RecordBatch};
    use arrow_ipc::writer::{IpcWriteOptions, StreamWriter};
    use arrow_schema::{DataType, Field, Schema};

    #[derive(Clone, Default)]
    struct SharedSink(Rc<RefCell<Vec<u8>>>);

    impl SharedSink {
        fn len(&self) -> usize {
            self.0.borrow().len()
        }
    }

    impl Write for SharedSink {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Int32,
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(Int32Array::from(vec![1, 2, 3]))]).unwrap()
    }

    fn zstd_options() -> IpcWriteOptions {
        IpcWriteOptions::default()
            .try_with_compression(Some(arrow_ipc::CompressionType::ZSTD))
            .unwrap()
    }

    pub(super) fn write_error_without_committed_batch() -> (String, usize, usize) {
        let batch = batch();
        let sink = SharedSink::default();
        let mut writer =
            StreamWriter::try_new_with_options(sink.clone(), &batch.schema(), zstd_options())
                .unwrap();
        let schema_bytes = sink.len();
        let error = writer
            .write(&batch)
            .expect_err("zstd record batch unexpectedly succeeded")
            .to_string();
        (error, schema_bytes, sink.len())
    }
}

#[cfg(not(feature = "zstd"))]
#[test]
fn zstd_without_logical_feature_reports_feature_disabled_without_committing_a_batch() {
    let (error, before, after) = writer_failure::write_error_without_committed_batch();
    assert!(error.contains("zstd IPC compression requires the zstd feature"));
    assert!(!error.contains("wasm32-unknown-unknown"));
    assert_eq!(before, after);
}

#[cfg(all(
    feature = "zstd",
    not(all(target_arch = "wasm32", target_os = "unknown"))
))]
#[test]
fn native_zstd_stream_round_trips() {
    let bytes = include_bytes!("data/wasm_zstd.arrow");
    let mut reader = StreamReader::try_new(Cursor::new(bytes), None).unwrap();
    assert_eq!(reader.schema().field(0).name(), "value");
    let batch = reader.next().unwrap().unwrap();
    assert_eq!(batch.num_rows(), 4096);
    assert!(reader.next().is_none());
}

#[cfg(all(feature = "zstd", target_arch = "wasm32", target_os = "unknown"))]
fn assert_target_unavailable(error: &str, operation: &str) {
    assert!(error.contains(&format!("cannot create Arrow IPC zstd {operation}")));
    assert!(error.contains("feature \"zstd\" is enabled"));
    assert!(error.contains("no backend is available"));
    assert!(error.contains("wasm32-unknown-unknown"));
}

#[cfg(all(feature = "zstd", target_arch = "wasm32", target_os = "unknown"))]
#[test]
fn wasm_zstd_writer_fails_before_committing_a_record_batch() {
    let (error, before, after) = writer_failure::write_error_without_committed_batch();
    assert_target_unavailable(&error, "compressor");
    assert_eq!(before, after);
}

#[cfg(all(feature = "zstd", target_arch = "wasm32", target_os = "unknown"))]
#[test]
fn wasm_reads_schema_before_zstd_record_batch_decode_fails() {
    let bytes = include_bytes!("data/wasm_zstd.arrow");
    let mut reader = StreamReader::try_new(Cursor::new(bytes), None).unwrap();
    assert_eq!(reader.schema().field(0).name(), "value");
    let error = reader
        .next()
        .expect("fixture must contain a record batch")
        .expect_err("zstd record batch decoding unexpectedly succeeded")
        .to_string();
    assert_target_unavailable(&error, "decompressor");
}
