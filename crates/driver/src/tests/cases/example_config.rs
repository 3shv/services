use {crate::tests, shared::addr};

/// Test that the example configuration file is valid by checking that the
/// driver does not crash when started with this file.
#[tokio::test]
#[ignore]
async fn test() {
    let example_config_file = std::env::current_dir().unwrap().join("example.toml");
    tests::setup()
        .config(example_config_file)
        .settlement_address(&addr!("9008D19f58AAbD9eD0D60971565AA8510560ab41"))
        .done()
        .await;
}
