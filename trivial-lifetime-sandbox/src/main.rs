use std::ffi::c_void;
use std::ptr;
use std::ptr::NonNull;

#[repr(C)]
#[derive(Debug)]
pub struct VTable {
    _reserved: *mut c_void,
    create: Option<extern "C-unwind" fn(this: *mut Plugin, out: *mut c_void) -> bool>,
    add_ref: Option<extern "C-unwind" fn(this: *mut Plugin) -> u32>,
    release: Option<extern "C-unwind" fn(this: *mut Plugin) -> u32>,
    get_refcnt: Option<extern "C-unwind" fn(this: *mut Plugin) -> u32>,
}

#[repr(C)]
#[derive(Debug)]
pub struct Plugin {
    vtbl_ptr: *const VTable,
    ref_cnt: u32,
}

fn main() {
    println!("Hello, world!");
}
