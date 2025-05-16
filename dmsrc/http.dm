#define RUSTG_HTTP_METHOD_GET "get"
#define RUSTG_HTTP_METHOD_PUT "put"
#define RUSTG_HTTP_METHOD_DELETE "delete"
#define RUSTG_HTTP_METHOD_PATCH "patch"
#define RUSTG_HTTP_METHOD_HEAD "head"
#define RUSTG_HTTP_METHOD_POST "post"
#define rustg_http_request_blocking(method, url, body, headers, options) RUSTG_CALL(RUST_G, "http_request_blocking")(method, url, body, headers, options)
#define rustg_http_request_async(method, url, body, headers, options) RUSTG_CALL(RUST_G, "http_request_async")(method, url, body, headers, options)
#define rustg_http_check_request(req_id) RUSTG_CALL(RUST_G, "http_check_request")(req_id)

// If you don't have the following proc in your codebase, you will need to uncomment it.
/***
/// Wrapper to let us runtime without killing the current proc, since CRASH only kills the exact proc it was called from
/proc/stack_trace(var/thing_to_crash)
	CRASH(thing_to_crash)
***/

/datum/http_request
	var/id
	var/in_progress = FALSE

	var/method
	var/body
	var/headers
	var/url
	/// If present, the request body will be read from this file.
	var/input_file = null
	/// If present, the response body will be saved to this file.
	var/output_file = null
	/// If present, request will timeout after this duration.
	var/timeout_seconds

	var/_raw_response

/datum/http_request/proc/prepare(method, url, body = "", list/headers, output_file, input_file, timeout_seconds)
	if (!length(headers))
		headers = ""
	else
		headers = json_encode(headers)

	src.method = method
	src.url = url
	src.body = body
	src.headers = headers
	src.input_file = input_file
	src.output_file = output_file
	src.timeout_seconds = timeout_seconds

/datum/http_request/proc/execute_blocking()
	_raw_response = rustg_http_request_blocking(method, url, body, headers, build_options())

/datum/http_request/proc/begin_async()
	if (in_progress)
		CRASH("Attempted to re-use a request object.")

	id = rustg_http_request_async(method, url, body, headers, build_options())

	if (isnull(text2num(id)))
		stack_trace("Proc error: [id]")
		_raw_response = "Proc error: [id]"
	else
		in_progress = TRUE

/datum/http_request/proc/build_options()
	. = json_encode(list(
		"input_filename" = (input_file ? input_file : null),
		"output_filename" = (output_file ? output_file : null),
		"timeout_seconds"=(timeout_seconds ? timeout_seconds : null)
	))

/datum/http_request/proc/is_complete()
	if (isnull(id))
		return TRUE

	if (!in_progress)
		return TRUE

	var/r = rustg_http_check_request(id)

	if (r == RUSTG_JOB_NO_RESULTS_YET)
		return FALSE
	else
		_raw_response = r
		in_progress = FALSE
		return TRUE

/datum/http_request/proc/into_response()
	var/datum/http_response/R = new()

	try
		var/list/L = json_decode(_raw_response)
		R.status_code = L["status_code"]
		R.headers = L["headers"]
		R.body = L["body"]
	catch
		R.errored = TRUE
		R.error = _raw_response

	return R

/datum/http_response
	/// The HTTP Status code - e.g., `"404"`
	var/status_code
	/// The response body - e.g., `{ "message": "No query results for xyz." }`
	var/body
	/// A list of headers - e.g., list("Content-Type" = "application/json").
	var/list/headers
	/// If the request errored, this will be TRUE.
	var/errored = FALSE
	/// If there was a 4xx/5xx error or the request failed to be sent, this will be the error message - e.g., `"HTTP error: 404"`
	/// If it's the former, `status_code` will be set.
	var/error
