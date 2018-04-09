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
        slice::from_raw_parts(argv, argc as usize)
            .into_iter()
            .map(|ptr| CStr::from_ptr(*ptr))
            .map(|cstr| cstr.to_string_lossy())
            .collect()
    }
}

pub fn byond_return<F, S>(inner: F) -> *const c_char
where
    F: FnOnce() -> Option<S>,
    S: Into<String>,
{
    match inner() {
        Some(str) => RETURN_STRING.with(|cell| {
            let cstr = CString::new(str.into()).expect("null in returned string!");
            let ptr = cstr.as_ptr();

            cell.set(cstr);
            ptr as *const c_char
        }),
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
            $crate::byond::byond_return(|| $body)
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

            $crate::byond::byond_return(|| $body)
        }
    };

    ($name:ident()! $body:block) => {
        byond_function!{ $name() {
            $body
            None as Option<String>
        } }
    };

    ($name:ident($($arg:ident),*)! $body:block) => {
        byond_function!{ $name($($arg),*) {
            $body
            None as Option<String>
        } }
    };
}
