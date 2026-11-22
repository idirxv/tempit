#[test]
fn cli_recorded_cases() {
    trycmd::TestCases::new()
        .register_bin("tempit", std::path::Path::new(env!("CARGO_BIN_EXE_tempit")))
        .env("TMPDIR", "/tmp")
        .insert_var("[VERSION]", env!("CARGO_PKG_VERSION"))
        .expect("valid version variable")
        .case("tests/cases/*.toml");
}
