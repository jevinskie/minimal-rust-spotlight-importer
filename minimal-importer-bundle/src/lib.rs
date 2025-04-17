// avoid pre-commit shebang confusion
#![feature(extern_types)]
#![feature(no_sanitize)]

use core::ffi::c_void;
use libc::{c_char, c_ulong};
use objc2::rc::Retained;
use objc2_core_foundation::{
    CFEqual, CFMutableDictionary, CFRetained, CFString, CFStringBuiltInEncodings,
    CFStringCreateWithBytes, CFStringEncoding, CFStringEncodings, CFType, CFUUID, CFUUIDBytes,
    kCFAllocatorDefault,
};
use std::{
    ffi::CStr,
    fs::File,
    io::{BufRead, BufReader, Read},
    ops::Deref,
    os::raw::c_void as void,
    path::Path,
    ptr,
};

type HRESULT = i32;
type ULONG = c_ulong;
type LPVOID = *mut c_void;
type REFIID = *const c_void;

/// This is the key Spotlight expects for the human-readable description.
const KMD_ITEM_DESCRIPTION: &str = "kMDItemDescription";

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MDImporterInterfaceStruct {
    _reserved: *mut c_void,
    query_interface:
        Option<unsafe extern "C-unwind" fn(*mut c_void, REFIID, *mut LPVOID) -> HRESULT>,
    add_ref: Option<unsafe extern "C-unwind" fn(*mut c_void) -> ULONG>,
    release: Option<unsafe extern "C-unwind" fn(*mut c_void) -> ULONG>,
    importer_import_data: Option<
        unsafe extern "C-unwind" fn(
            this: *mut c_void,
            attr: CFMutableDictionary,
            content_type_uti: CFString,
            path_to_file: CFString,
        ) -> bool,
    >,
}

unsafe extern "C-unwind" fn dummy_query_interface(
    _this: *mut c_void,
    _iid: REFIID,
    _out: *mut LPVOID,
) -> HRESULT {
    0
}

unsafe extern "C-unwind" fn dummy_add_ref(_this: *mut c_void) -> ULONG {
    1
}

unsafe extern "C-unwind" fn dummy_release(_this: *mut c_void) -> ULONG {
    1
}

unsafe extern "C-unwind" fn importer_import_data_impl(
    _this: *mut c_void,
    attr: CFMutableDictionary,
    _uti: CFString,
    path: CFString,
) -> bool {
    let cf_str = path;
    let path_str = cf_str.to_string();

    if let Ok(file) = File::open(&path_str) {
        let mut reader = BufReader::new(file);
        let mut first_line = String::new();
        if reader.read_line(&mut first_line).is_ok() {
            let desc_cfstr = CFString::from_str(first_line.trim());
            let key = CFString::from_str(KMD_ITEM_DESCRIPTION);
            // attr = desc_cfstr;
            // unsafe {objc2_core_foundation::CFDictionarySetValue(Some(&attr), key, &c_void::from(desc_cfst.)) };
            // unsafe { objc2_core_foundation::CFDictionarySetValue(attr as _, key.as_concrete_TypeRef() as _, desc_cfstr.as_concrete_TypeRef() as _) };
            return true;
        }
    }

    false
}

static mut INTERFACE: MDImporterInterfaceStruct = MDImporterInterfaceStruct {
    _reserved: ptr::null_mut(),
    query_interface: Some(dummy_query_interface),
    add_ref: Some(dummy_add_ref),
    release: Some(dummy_release),
    importer_import_data: Some(importer_import_data_impl),
};

fn AllocMetadataImporterPluginType(inFactoryID: CFUUID) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MetadataImporterPluginFactory(
    _type_id: *const c_void,
) -> *mut MDImporterInterfaceStruct {
    INTERFACE._reserved = ptr::null_mut();
    &raw mut INTERFACE
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ReturnCFString() -> *mut CFString {
    let s = "hellooo";
    let r = unsafe {
        CFStringCreateWithBytes(
            kCFAllocatorDefault,
            s.as_ptr(),
            s.len() as isize,
            CFStringBuiltInEncodings::EncodingUTF8.0,
            false,
        )
    }
    .unwrap();
    println!("r: {r:#?} {r:#} {r:?}");
    let rx = unsafe { Retained::retain(Retained::into_raw(r.into())) }.unwrap();
    let r2 = Retained::into_raw(rx.into());
    println!("r2: {r2:#?} {r2:?}");
    r2
}
