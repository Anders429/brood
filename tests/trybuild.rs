macro_rules! trybuild_test {
    ($test_name:ident) => {
        #[test]
        #[rustversion::attr(not(nightly), ignore)]
        fn $test_name() {
            trybuild::TestCases::new().compile_fail(concat!("tests/trybuild/", stringify!($test_name), "/*.rs"));
        }
    }
}

trybuild_test!(entities);
#[cfg(feature = "parallel")]
trybuild_test!(stages);