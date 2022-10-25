#define rustg_dmi_strip_metadata(fname) RGCALL(RUST_G, "dmi_strip_metadata")(fname)
#define rustg_dmi_create_png(path, width, height, data) RGCALL(RUST_G, "dmi_create_png")(path, width, height, data)
#define rustg_dmi_resize_png(path, width, height, resizetype) RGCALL(RUST_G, "dmi_resize_png")(path, width, height, resizetype)
