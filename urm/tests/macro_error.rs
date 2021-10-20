#[test]
fn macro_error() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/macro_error/*.rs");
}
