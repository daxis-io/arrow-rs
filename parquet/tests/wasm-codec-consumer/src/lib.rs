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
    use bytes::Bytes;
    use parquet::file::reader::{FileReader, SerializedFileReader};
    use wasm_bindgen_test::wasm_bindgen_test;

    fn reader() -> SerializedFileReader<Bytes> {
        let bytes = Bytes::from_static(include_bytes!("../../data/wasm_zstd.parquet"));
        SerializedFileReader::new(bytes).unwrap()
    }

    fn page_decode_error(reader: &SerializedFileReader<Bytes>) -> String {
        match reader.get_row_iter(None) {
            Err(error) => error.to_string(),
            Ok(mut rows) => match rows.next() {
                Some(Err(error)) => error.to_string(),
                Some(Ok(_)) => panic!("zstd page decoding unexpectedly succeeded"),
                None => panic!("fixture unexpectedly contained no rows"),
            },
        }
    }

    #[wasm_bindgen_test]
    fn reads_metadata_before_reporting_target_unavailable_zstd() {
        let reader = reader();
        assert_eq!(reader.metadata().file_metadata().num_rows(), 3);
        assert_eq!(
            reader
                .metadata()
                .file_metadata()
                .schema_descr()
                .num_columns(),
            1
        );

        let error = page_decode_error(&reader);
        assert!(
            error.contains("cannot create Parquet zstd codec"),
            "{error}"
        );
        assert!(error.contains("feature \"zstd\" is enabled"), "{error}");
        assert!(error.contains("no backend is available"), "{error}");
        assert!(error.contains("wasm32-unknown-unknown"), "{error}");
    }
}
