extern crate html2text;

byond_fn! { render_html(data) {
    Some(html2text::from_read(&mut data.as_bytes(), 2048))
} }
