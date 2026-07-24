use bytes::Bytes;
use parquet::file::reader::{FileReader, SerializedFileReader};

fn reader() -> SerializedFileReader<Bytes> {
    let bytes = Bytes::from_static(include_bytes!("data/wasm_zstd.parquet"));
    SerializedFileReader::new(bytes).unwrap()
}

#[cfg(any(
    not(feature = "zstd"),
    all(feature = "zstd", target_arch = "wasm32", target_os = "unknown")
))]
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

#[cfg(not(feature = "zstd"))]
#[test]
fn zstd_without_logical_feature_reports_feature_disabled() {
    let error = page_decode_error(&reader());
    assert!(error.contains("Disabled feature at compile time: zstd"));
    assert!(!error.contains("wasm32-unknown-unknown"));
}

#[cfg(all(
    feature = "zstd",
    not(all(target_arch = "wasm32", target_os = "unknown"))
))]
#[test]
fn native_zstd_backend_round_trips() {
    let rows = reader()
        .get_row_iter(None)
        .unwrap()
        .collect::<parquet::errors::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(rows.len(), 3);
}

#[cfg(all(feature = "zstd", target_arch = "wasm32", target_os = "unknown"))]
fn assert_target_unavailable(error: &str) {
    assert!(error.contains("cannot create Parquet zstd codec"));
    assert!(error.contains("feature \"zstd\" is enabled"));
    assert!(error.contains("no backend is available"));
    assert!(error.contains("wasm32-unknown-unknown"));
}

#[cfg(all(feature = "zstd", target_arch = "wasm32", target_os = "unknown"))]
#[test]
fn wasm_zstd_reports_target_unavailable() {
    assert_target_unavailable(&page_decode_error(&reader()));
}

#[cfg(all(feature = "zstd", target_arch = "wasm32", target_os = "unknown"))]
#[test]
fn wasm_reads_zstd_footer_and_schema_before_page_decode_fails() {
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
    assert_target_unavailable(&error);
}
