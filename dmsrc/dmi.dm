#define rustg_dmi_strip_metadata(fname) RUSTG_CALL(RUST_G, "dmi_strip_metadata")(fname)
#define rustg_dmi_create_png(path, width, height, data) RUSTG_CALL(RUST_G, "dmi_create_png")(path, width, height, data)
#define rustg_dmi_resize_png(path, width, height, resizetype) RUSTG_CALL(RUST_G, "dmi_resize_png")(path, width, height, resizetype)
/**
 * input: must be a path, not an /icon; you have to do your own handling if it is one, as icon objects can't be directly passed to rustg.
 *
 * output: json_encode'd list. json_decode to get a flat list with icon states in the order they're in inside the .dmi
 */
#define rustg_dmi_icon_states(fname) RUSTG_CALL(RUST_G, "dmi_icon_states")(fname)
/**
 * input:
 * fname: must be a path, not an /icon; you have to do your own handling if it is one, as icon objects can't be directly passed to rustg.
 * output: path to output the css to
 * namemap: must be a json_encode'd list of {'icon_state': 'desired name'} or ""
 *
 * output: an entire css file ready to ship to the client. the classes will be `.{path_name}.{icon_state}` so make sure your icon states are good already
 */
#define rustg_dmi_convert_to_svgcss(fname, output, namemap) RUSTG_CALL(RUST_G, "dmi_convert_to_svgcss")(fname, output, namemap)
/**
 * input:
 * fname: path you want to write to
 */
#define rustg_dmi_start_svg_symbols(fname) RUSTG_CALL(RUST_G, "dmi_start_svg_symbols")(fname)
/**
 * MUST CALL rustg_dmi_start_svg_symbols and rustg_dmi_end_svg_symbols
 * 
 * input:
 * fname: must be a path, not an /icon; you have to do your own handling if it is one, as icon objects can't be directly passed to rustg.
 * output: path to output the svg <symbol> list to
 * namemap: must be a json_encode'd list of {'icon_state': 'desired name'} or ""
 *
 * output: ready-to-ship svg file
 */
#define rustg_dmi_convert_to_svg_symbols(fname, output, namemap) RUSTG_CALL(RUST_G, "dmi_convert_to_svg_symbols")(fname, output, namemap)
/**
 * input:
 * fname: path you want to write to
 */
#define rustg_dmi_end_svg_symbols(fname) RUSTG_CALL(RUST_G, "dmi_end_svg_symbols")(fname)