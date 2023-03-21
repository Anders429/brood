#![cfg(not(skip_trybuild))]

macro_rules! trybuild_test {
    ($test_name:ident) => {
        #[rustversion::attr(not(nightly), ignore)]
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
