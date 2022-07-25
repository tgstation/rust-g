
#define rustg_dmi_strip_metadata(fname) call(RUST_G, "dmi_strip_metadata")(fname)
#define rustg_dmi_create_png(path, width, height, data) call(RUST_G, "dmi_create_png")(path, width, height, data)
#define rustg_dmi_resize_png(path, width, height, resizetype) call(RUST_G, "dmi_resize_png")(path, width, height, resizetype)
