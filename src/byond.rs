use std::cell::Cell;
use std::ffi::{CStr, CString};
use std::slice;

use libc::{c_char, c_int};

thread_local! {
    static RETURN_STRING: Cell<CString> = Cell::new(Default::default());
}

pub fn parse_args(argc: c_int, argv: *const *const c_char) -> Vec<String> {
    let args: Vec<&CStr> = unsafe {
        slice::from_raw_parts(argv, argc as usize)
            .into_iter()
            .map(|ptr| CStr::from_ptr(*ptr))
            .collect()
    };

    args.into_iter()
        .map(|cstr| cstr.to_string_lossy())
        .map(|str| str.into_owned())
        .collect()
}

pub fn return_string(string: Option<String>) -> *const c_char {
    let cstring = match string {
        Some(msg) => CString::new(msg).expect("null in returned string!"),
        None => CString::new("").unwrap(),
    };
    let ptr = cstring.as_ptr();

    RETURN_STRING.with(|cell| {
        cell.set(cstring);
    });

    ptr as *const c_char
}

#[macro_export]
macro_rules! byond_function {
    ($name:ident() $body:block) => {
        #[no_mangle]
        pub extern "C" fn $name(
            _argc: ::libc::c_int, _argv: *const *const ::libc::c_char
        ) -> *const ::libc::c_char {
            $crate::byond::return_string((|| $body)())
        }
    };

    ($name:ident($($arg:ident),*) $body:block) => {
        #[no_mangle]
        pub extern "C" fn $name(
            _argc: ::libc::c_int, _argv: *const *const ::libc::c_char
        ) -> *const ::libc::c_char {
            let __args = $crate::byond::parse_args(_argc, _argv);

            let mut __argn = 0;
            $(
                let $arg = &__args[__argn];
                __argn += 1;
            )*

            $crate::byond::return_string((|| $body)())
        }
    };
}

