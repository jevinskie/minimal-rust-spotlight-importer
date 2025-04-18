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
use std::ops::Deref;
use std::ptr;
use std::ptr::NonNull;

/// This is the key Spotlight expects for the human-readable description.
const KMD_ITEM_DESCRIPTION: &str = "kMDItemDescription";

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MDImporterInterfaceStruct {
    _reserved: *mut c_void,
    query_interface: Option<
        unsafe extern "C-unwind" fn(
            this: *mut MetadataImporterPluginType,
            iid: REFIID,
            out: *mut LPVOID,
        ) -> HRESULT,
    >,
    add_ref: Option<unsafe extern "C-unwind" fn(this: *mut MetadataImporterPluginType) -> ULONG>,
    release: Option<unsafe extern "C-unwind" fn(this: *mut MetadataImporterPluginType) -> ULONG>,
    importer_import_data: Option<
        unsafe extern "C-unwind" fn(
            this: *mut MetadataImporterPluginType,
            attr: *mut CFMutableDictionary,
            content_type_uti: *mut CFString,
            path_to_file: *mut CFString,
        ) -> bool,
    >,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MetadataImporterPluginType {
    conduitInterface: *mut MDImporterInterfaceStruct,
    factoryID: *mut CFUUID,
    refCount: u32,
}

unsafe extern "C-unwind" fn com_query_interface(
    this: *mut MetadataImporterPluginType,
    iid: REFIID,
    out: *mut LPVOID,
) -> HRESULT {
    0
}

unsafe extern "C-unwind" fn com_add_ref(this: *mut MetadataImporterPluginType) -> ULONG {
    let pt = &mut unsafe { *this };
    if pt.refCount < 1 {
        panic!("ref count underflow");
    }
    pt.refCount += 1;
    pt.refCount as ULONG
}

unsafe extern "C-unwind" fn com_release(this: *mut MetadataImporterPluginType) -> ULONG {
    let pt = &mut unsafe { *this };
    if pt.refCount < 1 {
        panic!("ref count underflow");
    }
    pt.refCount -= 1;
    if pt.refCount != 0 {
        pt.refCount as ULONG
    } else {
        let fuuid = unsafe { CFRetained::from_raw(NonNull::new(pt.factoryID).unwrap()) };
        pt.factoryID = ptr::null_mut();
        let ptb = unsafe { Box::from_raw(this) };
        println!("com_release drop this: {this:#?} pt: {pt:#?} ptb: {ptb:#?}");
        drop(ptb);
        println!("com_release drop fuuid: {fuuid:#?}");
        0
    }
}

unsafe extern "C-unwind" fn com_importer_import_data(
    this: *mut MetadataImporterPluginType,
    attr: *mut CFMutableDictionary,
    uti: *mut CFString,
    path: *mut CFString,
) -> bool {
    println!("com_importer_import_data this: {this:#?}");
    let path_cfstr = unsafe { CFRetained::retain(NonNull::new(path).unwrap()) };
    let path_str = path_cfstr.to_string();
    println!("com_importer_import_data path: {path_str}");
    let attro = unsafe { CFRetained::retain(NonNull::new(attr).unwrap()) };
    println!("com_importer_import_data attr: {attro:#?}");
    let utio = unsafe { CFRetained::retain(NonNull::new(uti).unwrap()) };
    println!("com_importer_import_data uti: {utio:#?}");

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

fn MetadataImporterPluginFactoryUUID() -> CFRetained<CFUUID> {
    unsafe {
        CFUUIDGetConstantUUIDWithBytes(
            kCFAllocatorDefault,
            0x93,
            0x36,
            0xd6,
            0xdb,
            0x18,
            0xf0,
            0x46,
            0x15,
            0x89,
            0xe4,
            0x7a,
            0x12,
            0x34,
            0xbd,
            0xaa,
            0x7b,
        )
    }
    .unwrap()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn MetadataImporterPluginFactory(
    allocator: *mut CFAllocator,
    inFactoryID: *mut CFUUID,
) -> *mut MetadataImporterPluginType {
    println!("passed allocator: {allocator:#?}");
    println!("passed uuid ptr: {inFactoryID:#?}");
    let uuid = unsafe { CFRetained::retain(NonNull::new(inFactoryID).unwrap()) };
    let importer_uuid = kMDImporterTypeID();
    println!("passed uuid: {uuid:#?} importer uuid: {importer_uuid:#?}");
    if uuid == importer_uuid {
        let s = MDImporterInterfaceStruct {
            _reserved: ptr::null_mut(),
            query_interface: Some(com_query_interface),
            add_ref: Some(com_add_ref),
            release: Some(com_release),
            importer_import_data: Some(com_importer_import_data),
        };
        let ifu = MetadataImporterPluginFactoryUUID();
        unsafe { CFPlugInAddInstanceForFactory(Some(&ifu)) };
        let ifu_ptr = CFRetained::into_raw(ifu).as_ptr();
        let mut br = Box::new(s);
        let ptr = Box::as_mut_ptr(&mut br);
        mem::forget(br);
        let pt = MetadataImporterPluginType {
            conduitInterface: ptr,
            factoryID: ifu_ptr,
            refCount: 1,
        };
        let mut bp = Box::new(pt);
        let bp_ptr = Box::as_mut_ptr(&mut bp);
        mem::forget(bp);
        bp_ptr
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
