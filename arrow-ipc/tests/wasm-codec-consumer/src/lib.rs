// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements. See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership. The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. See the License for the
// specific language governing permissions and limitations
// under the License.

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::io::{Cursor, Write};
    use std::rc::Rc;
    use std::sync::Arc;

    use arrow_array::{Int32Array, RecordBatch};
    use arrow_ipc::reader::StreamReader;
    use arrow_ipc::writer::{IpcWriteOptions, StreamWriter};
    use arrow_schema::{DataType, Field, Schema};
    use wasm_bindgen_test::wasm_bindgen_test;

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

    fn assert_target_unavailable(error: &str, operation: &str) {
        assert!(error.contains(&format!("cannot create Arrow IPC zstd {operation}")));
        assert!(error.contains("feature \"zstd\" is enabled"), "{error}");
        assert!(error.contains("no backend is available"), "{error}");
        assert!(error.contains("wasm32-unknown-unknown"), "{error}");
    }

    #[wasm_bindgen_test]
    fn writer_fails_without_committing_a_record_batch() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Int32,
            false,
        )]));
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(Int32Array::from(vec![1, 2, 3]))]).unwrap();
        let options = IpcWriteOptions::default()
            .try_with_compression(Some(arrow_ipc::CompressionType::ZSTD))
            .unwrap();
        let sink = SharedSink::default();
        let mut writer =
            StreamWriter::try_new_with_options(sink.clone(), &batch.schema(), options).unwrap();
        let schema_bytes = sink.len();
        let error = writer
            .write(&batch)
            .expect_err("zstd record batch unexpectedly succeeded")
            .to_string();

        assert_target_unavailable(&error, "compressor");
        assert_eq!(schema_bytes, sink.len());
    }

    #[wasm_bindgen_test]
    fn reader_consumes_schema_before_reporting_target_unavailable() {
        let bytes = include_bytes!("../../data/wasm_zstd.arrow");
        let mut reader = StreamReader::try_new(Cursor::new(bytes), None).unwrap();
        assert_eq!(reader.schema().field(0).name(), "value");
        let error = reader
            .next()
            .expect("fixture must contain a record batch")
            .expect_err("zstd record batch decoding unexpectedly succeeded")
            .to_string();

        assert_target_unavailable(&error, "decompressor");
    }
}
