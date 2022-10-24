byond_fn!(
    fn panic_test() {
        panic!("oh no");

        #[allow(unreachable_code)]
        Some("what".to_owned())
    }
);
