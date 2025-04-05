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
 * Flattens a list of pixel layers into a single layer by blending the colors of each layer.
 * Colors are blended with the first layer on the bottom, and each successive layer on top of the previous one.
 *
 * `data` is a a json_encoded list of lists of hexadecimal color strings (null is treated as #00000000)
 *
 * Returns a json_encode'd list as long as the longest layer in `data`
 */
#define rustg_dmi_flatten_layers(data) RUSTG_CALL(RUST_G, "dmi_flatten_layers")(data)

/**
 * Generates a dmi file at the specified path.
 *
 * "data" format:
 * list(
 *      "width": number
 *      "height": number
 *      "states": [STATE][]
 * )
 *
 * STATE format:
 * list(
 *     "name": string
 *     "dirs": 1 | 4 | 8
 *     "delay"?: number[]
 *     "rewind": boolean
 *     "movement": boolean
 *     "loop"?: number
 *     "pixels": string[]
 * )
 *
 * STATE["pixels"] is a list of pixels flattened to one dimension.
 * A single pixel is represented by a hexadecimal color string with an optional alpha channel (null is treated as #00000000).
 * A single row consists of `width` pixels.
 * A single dir-frame consists of `height` rows.
 * A single frame consists of `dirs` dir-frames, in the order [SOUTH, NORTH, EAST, WEST, SOUTHEAST, SOUTHWEST, NORTHEAST, NORTHWEST].
 * A single state consists of one frame for each number in `delay`, or a single frame if `delay` is not present.
 */
#define rustg_dmi_create_dmi(path, data) RUSTG_CALL(RUST_G, "dmi_create_dmi")(path, data)
