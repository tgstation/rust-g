use uuid::Uuid;

byond_fn!(
    fn uuid_v4() {
        Some(Uuid::new_v4().to_string())
    }
);

byond_fn!(
    fn uuid_v7() {
        Some(Uuid::now_v7().to_string())
    }
);

byond_fn!(
    fn cuid2() {
        Some(cuid2::create_id())
    }
);

byond_fn!(
    fn cuid2_len(length) {
        let length = length.parse::<u16>().ok()?;
        Some(
            cuid2::CuidConstructor::new()
                .with_length(length)
                .create_id()
        )
    }
);
