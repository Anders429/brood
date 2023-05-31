#![cfg(not(skip_trybuild))]

#[test]
fn check_msrv() {
    // If this test fails, the MSRV needs to be updated both here and in the `trybuild_test!` macro
    // definition. This ensures that the trybuild tests are run on the MSRV even when the MSRV is
    // updated.
    assert_eq!(env!("CARGO_PKG_RUST_VERSION"), "1.65.0")
}

macro_rules! trybuild_test {
    ($test_name:ident) => {
        #[rustversion::attr(not(stable(1.65)), ignore)]
        #[test]
        fn $test_name() {
            trybuild::TestCases::new().compile_fail(concat!(
                "tests/trybuild/",
                stringify!($test_name),
                "/*.rs"
            ));
        }
    };
}

trybuild_test!(entities);
trybuild_test!(entity);
trybuild_test!(registry);
trybuild_test!(resources);
trybuild_test!(result);
#[cfg(feature = "rayon")]
trybuild_test!(schedule);
trybuild_test!(views);
