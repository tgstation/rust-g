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
 * The below functions involve dmi metadata represented in the following format:
 * list(
 *     "width": number,
 *     "height": number,
 *     "states": list([STATE_DATA], ...)
 * )
 *
 * STATE_DATA format:
 * list(
 *     "name": string,
 *     "dirs": 1 | 4 | 8,
 *     "delays"?: list(number, ...),
 *     "rewind"?: TRUE | FALSE,
 *     "movement"?: TRUE | FALSE,
 *     "loop"?: number
 * )
 */

/**
 * Get the dmi metadata of the file located at `fname`.
 * Returns a json_encode'd list in the metadata format listed above, or an error message.
 */
#define rustg_dmi_read_metadata(fname) RUSTG_CALL(RUST_G, "dmi_read_metadata")(fname)
/**
 * Inject dmi metadata into a png file located at `path`.
 * `metadata` must be a json_encode'd list in the metadata format listed above.
 */
#define rustg_dmi_inject_metadata(path, metadata) RUSTG_CALL(RUST_G, "dmi_inject_metadata")(path, metadata)
