// avoid pre-commit shebang confusion
#![feature(extern_types)]
#![feature(no_sanitize)]
#![feature(box_as_ptr)]

use core::ffi::c_void;
use objc2_core_foundation::{
    CFAllocator, CFMutableDictionary, CFPlugInAddInstanceForFactory, CFRetained, CFString,
    CFStringBuiltInEncodings, CFStringCreateWithBytes, CFUUID, CFUUIDGetConstantUUIDWithBytes,
    HRESULT, LPVOID, REFIID, ULONG, kCFAllocatorDefault,
};
use std::mem;
use std::ptr;
use std::ptr::NonNull;

/// This is the key Spotlight expects for the human-readable description.
const KMD_ITEM_DESCRIPTION: &str = "kMDItemDescription";

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MDImporterInterfaceStruct {
    _reserved: *mut c_void,
    query_interface: Option<
        unsafe extern "C-unwind" fn(this: *mut c_void, iid: REFIID, out: *mut LPVOID) -> HRESULT,
    >,
    add_ref: Option<unsafe extern "C-unwind" fn(this: *mut c_void) -> ULONG>,
    release: Option<unsafe extern "C-unwind" fn(this: *mut c_void) -> ULONG>,
    importer_import_data: Option<
        unsafe extern "C-unwind" fn(
            this: *mut c_void,
            attr: *mut CFMutableDictionary,
            content_type_uti: *mut CFString,
            path_to_file: *mut CFString,
        ) -> bool,
    >,
}

unsafe extern "C-unwind" fn dummy_query_interface(
    this: *mut c_void,
    iid: REFIID,
    out: *mut LPVOID,
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
    this: *mut c_void,
    attr: *mut CFMutableDictionary,
    uti: *mut CFString,
    path: *mut CFString,
) -> bool {
    let path_cfstr = unsafe { CFRetained::<CFString>::retain(NonNull::new(path).unwrap()) };
    let path_str = path_cfstr.to_string();
    println!("importer_import_data_impl path: {path_str}");

    // if let Ok(file) = File::open(&path_str) {
    //     let mut reader = BufReader::new(file);
    //     let mut first_line = String::new();
    //     if reader.read_line(&mut first_line).is_ok() {
    //         let desc_cfstr = CFString::from_str(first_line.trim());
    //         let key = CFString::from_str(KMD_ITEM_DESCRIPTION);
    //         // attr = desc_cfstr;
    //         // unsafe {objc2_core_foundation::CFDictionarySetValue(Some(&attr), key, &c_void::from(desc_cfst.)) };
    //         // unsafe { objc2_core_foundation::CFDictionarySetValue(attr as _, key.as_concrete_TypeRef() as _, desc_cfstr.as_concrete_TypeRef() as _) };
    //         return true;
    //     }
    // }

    false
}

fn kMDImporterTypeID() -> CFRetained<CFUUID> {
    unsafe {
        CFUUIDGetConstantUUIDWithBytes(
            kCFAllocatorDefault,
            0x8B,
            0x08,
            0xC4,
            0xBF,
            0x41,
            0x5B,
            0x11,
            0xD8,
            0xB3,
            0xF9,
            0x00,
            0x03,
            0x93,
            0x67,
            0x26,
            0xFC,
        )
    }
    .unwrap()
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
pub unsafe extern "C-unwind" fn MetadataImporterPluginFactory(
    allocator: *mut CFAllocator,
    inFactoryID: *mut CFUUID,
) -> *mut MDImporterInterfaceStruct {
    println!("passed allocator: {allocator:#?}");
    println!("passed uuid ptr: {inFactoryID:#?}");
    let uuid = unsafe { CFRetained::<CFUUID>::retain(NonNull::new(inFactoryID).unwrap()) };
    let importer_uuid = kMDImporterTypeID();
    println!("passed uuid: {uuid:#?} importer uuid: {importer_uuid:#?}");
    if uuid == importer_uuid {
        let s = MDImporterInterfaceStruct {
            _reserved: ptr::null_mut(),
            query_interface: Some(dummy_query_interface),
            add_ref: Some(dummy_add_ref),
            release: Some(dummy_release),
            importer_import_data: Some(importer_import_data_impl),
        };
        let mut br = Box::<MDImporterInterfaceStruct>::new(s);
        let ptr: *mut MDImporterInterfaceStruct =
            Box::<MDImporterInterfaceStruct>::as_mut_ptr(&mut br);
        mem::forget(br);
        ptr
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn ReturnCFString() -> *mut CFString {
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
    println!("ReturnCFString r: {r}");
    CFRetained::as_ptr(&r).as_ptr()
}
