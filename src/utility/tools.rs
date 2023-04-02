
use std::ffi::{c_char, CStr};
//use std::string::String;
pub fn char_array_to_string(raw_string_array: &[c_char]) -> String
{
    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };
    raw_string
        .to_str()
        .expect("Failed to convert char array to string.")
        .to_owned()
}