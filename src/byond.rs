use std::borrow::Cow;
use std::cell::Cell;
use std::ffi::{CStr, CString};
use std::slice;

use libc::{c_char, c_int};

static EMPTY_STRING: &[c_char; 1] = &[0];
thread_local! {
    static RETURN_STRING: Cell<CString> = Cell::new(CString::default());
}

pub fn parse_args<'a>(argc: c_int, argv: *const *const c_char) -> Vec<Cow<'a, str>> {
    unsafe {
        slice::from_raw_parts(argv, argc as usize).into_iter()
            .map(|ptr| CStr::from_ptr(*ptr))
            .map(|cstr| cstr.to_string_lossy())
            .collect()
    }
}

pub fn return_string(string: Option<String>) -> *const c_char {
    match string {
        Some(string) =>  {
            RETURN_STRING.with(|cell| {
                let cstring = CString::new(string).expect("null in returned string!");
                let ptr = cstring.as_ptr();

                cell.set(cstring);
                ptr as *const c_char
            })
        },
        None => EMPTY_STRING as *const c_char,
    }
}

#[macro_export]
macro_rules! byond_function {
    ($name:ident() $body:block) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
            _argc: ::libc::c_int, _argv: *const *const ::libc::c_char
        ) -> *const ::libc::c_char {
            $crate::byond::return_string((|| $body)())
        }
    };

    ($name:ident($($arg:ident),*) $body:block) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
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

