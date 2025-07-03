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
    fn cuid1() {
        cuid::cuid1().ok()
    }
);

byond_fn!(
    fn cuid2() {
        Some(cuid::cuid2())
    }
);

byond_fn!(
    fn cuid2_len(length) {
        let length = length.parse::<u16>().ok()?;
        Some(
            cuid::Cuid2Constructor::new()
                .with_length(length)
                .create_id()
        )
    }
);
