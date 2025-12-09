use std::ffi::CString;

struct Utf8Pointer {
    c_strings: Vec<CString>,
    pointers: Vec<*const i8>, // points to cstrings above
}

impl Utf8Pointer {
    pub fn new<S: AsRef<&str>>(string_list: &[S]) -> Self {
        let c_strings: Vec<CString> = string_list
            .iter()
            .map(|s| CString::new(s.as_ref()).unwrap())
            .collect();

        let pointers: Vec<*const i8> = c_strings.iter().map(|c| c.as_ptr()).collect();

        Self {
            c_strings,
            pointers,
        }
    }

    pub fn as_ptr(&self) -> *const *const i8 {
        self.pointers.as_ptr()
    }
}
