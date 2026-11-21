#[test]
fn cli_recorded_cases() {
    trycmd::TestCases::new()
        .register_bin("tempit", std::path::Path::new(env!("CARGO_BIN_EXE_tempit")))
        .case("tests/cases/*.toml");
}
