// avoid pre-commit shebang confusion
// #![feature(extern_types)]
// #![feature(no_sanitize)]
// #![feature(box_as_ptr)]

use log::{LevelFilter, info};
use objc2_core_foundation::{
    CFAllocator, CFMutableDictionary, CFPlugIn, CFRetained, CFString, CFUUID, HRESULT, LPVOID,
    REFIID, ULONG, kCFAllocatorDefault,
};
use oslog::OsLogger;
use std::ffi::c_void;
use std::ptr;
use std::ptr::NonNull;

/// This is the key Spotlight expects for the human-readable description.
const KMD_ITEM_DESCRIPTION: &str = "kMDItemDescription";

fn kMDImporterTypeID() -> CFRetained<CFUUID> {
    CFUUID::constant_uuid_with_bytes(
        unsafe { kCFAllocatorDefault },
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
    .unwrap()
}

fn kMDImporterInterfaceID() -> CFRetained<CFUUID> {
    CFUUID::constant_uuid_with_bytes(
        unsafe { kCFAllocatorDefault },
        0x6E,
        0xBC,
        0x27,
        0xC4,
        0x89,
        0x9C,
        0x11,
        0xD8,
        0x84,
        0xAE,
        0x00,
        0x03,
        0x93,
        0x67,
        0x26,
        0xFC,
    )
    .unwrap()
}

fn MetadataImporterPluginFactoryUUID() -> CFRetained<CFUUID> {
    CFUUID::constant_uuid_with_bytes(
        unsafe { kCFAllocatorDefault },
        0xd8,
        0x78,
        0x57,
        0xf7,
        0xb0,
        0xc0,
        0x4c,
        0x70,
        0x9b,
        0x8f,
        0x2e,
        0x3d,
        0x8e,
        0x55,
        0x19,
        0x8c,
    )
    .unwrap()
}

fn IUnknownUUID() -> CFRetained<CFUUID> {
    CFUUID::constant_uuid_with_bytes(
        unsafe { kCFAllocatorDefault },
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0xC0,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x46,
    )
    .unwrap()
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MDImporterInterfaceStruct {
    _reserved: *mut c_void,
    query_interface: Option<
        extern "C-unwind" fn(
            this: *mut MetadataImporterPluginType,
            iid: REFIID,
            out: *mut LPVOID,
        ) -> HRESULT,
    >,
    add_ref: Option<extern "C-unwind" fn(this: *mut MetadataImporterPluginType) -> ULONG>,
    release: Option<extern "C-unwind" fn(this: *mut MetadataImporterPluginType) -> ULONG>,
    importer_import_data: Option<
        extern "C-unwind" fn(
            this: *mut MetadataImporterPluginType,
            attr: *mut CFMutableDictionary,
            content_type_uti: *mut CFString,
            path_to_file: *mut CFString,
        ) -> bool,
    >,
}

unsafe impl Send for MDImporterInterfaceStruct {}
unsafe impl Sync for MDImporterInterfaceStruct {}

impl AsRef<MDImporterInterfaceStruct> for MDImporterInterfaceStruct {
    fn as_ref(&self) -> &MDImporterInterfaceStruct {
        &self
    }
}

impl AsMut<MDImporterInterfaceStruct> for MDImporterInterfaceStruct {
    fn as_mut(&mut self) -> &mut MDImporterInterfaceStruct {
        self
    }
}

impl MDImporterInterfaceStruct {
    pub fn as_ptr(&mut self) -> *mut MDImporterInterfaceStruct {
        self
    }
    pub fn query_interface_safe(
        &self,
        handle: &mut MetadataImporterPluginType,
        iid: CFRetained<CFUUID>,
        out: *mut LPVOID,
    ) -> HRESULT {
        println!("query_interface_safe: handle: {handle:#?} iid: {iid:#?} out: {out:#?}");
        if iid == kMDImporterInterfaceID() || iid == IUnknownUUID() {
            handle.add_ref();
            0 // S_OK
        } else {
            unsafe { *out = ptr::null_mut() };
            1 // S_FALSE
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MetadataImporterPluginType {
    conduitInterface: *const MDImporterInterfaceStruct,
    factoryID: *mut CFUUID,
    refCount: u32,
}

impl MetadataImporterPluginType {
    pub fn intf(&self) -> &MDImporterInterfaceStruct {
        unsafe { self.conduitInterface.as_ref() }.unwrap()
    }
    // pub fn intf_mut(&mut self) -> &mut MDImporterInterfaceStruct {
    //     unsafe { self.conduitInterface.as_ref() }.unwrap().as_mut()
    // }
    pub fn factory_id(&self) -> CFRetained<CFUUID> {
        let nnfid = NonNull::new(self.factoryID).unwrap();
        unsafe { CFRetained::from_raw(nnfid) }
    }
    pub fn query_interface(&mut self, iid: CFRetained<CFUUID>, out: *mut LPVOID) -> HRESULT {
        let im = unsafe { self.conduitInterface.as_ref() }.unwrap();
        im.query_interface_safe(self, iid, out)
    }
    pub fn add_ref(&mut self) -> ULONG {
        self.intf().add_ref.unwrap()(self)
    }
}

extern "C-unwind" fn com_query_interface(
    this: *mut MetadataImporterPluginType,
    iid: REFIID,
    out: *mut LPVOID,
) -> HRESULT {
    let t = unsafe { this.as_mut() }.unwrap();
    let iuuid = unsafe { CFUUID::from_uuid_bytes(kCFAllocatorDefault, iid) }.unwrap();
    t.query_interface(iuuid, out)
}

extern "C-unwind" fn com_add_ref(this: *mut MetadataImporterPluginType) -> ULONG {
    let pt = &mut unsafe { *this };
    println!("com_add_ref this: {this:#?} pt: {pt:#?}");
    if pt.refCount < 1 {
        panic!("ref count underflow");
    }
    pt.refCount = pt.refCount.checked_add(1).unwrap();
    println!("com_add_ref end this: {this:#?} pt: {pt:#?}");
    pt.refCount as ULONG
}

fn com_add_ref_safe(this: &mut MetadataImporterPluginType) -> ULONG {
    println!("com_add_ref_safe begin this: {this:#?}");
    if this.refCount < 1 {
        panic!("ref count underflow");
    }
    this.refCount = this.refCount.checked_add(1).unwrap();
    println!("com_add_ref_safe end this: {this:#?}");
    this.refCount as ULONG
}

extern "C-unwind" fn com_release(this: *mut MetadataImporterPluginType) -> ULONG {
    let pt = &mut unsafe { *this };
    println!("com_release this: {this:#?} pt: {pt:#?}");
    if pt.refCount < 1 {
        panic!("ref count underflow");
    }
    pt.refCount = pt.refCount.checked_sub(1).unwrap();
    if pt.refCount != 0 {
        println!("com_release end (no-dealloc) this: {this:#?} pt: {pt:#?}");
        pt.refCount as ULONG
    } else {
        println!("com_release end (dealloc) this: {this:#?} pt: {pt:#?}");
        let fuuid = unsafe { CFRetained::from_raw(NonNull::new(pt.factoryID).unwrap()) };
        pt.factoryID = ptr::null_mut();
        let ptb = unsafe { Box::from_raw(this) };
        println!("com_release drop ptb: {this:#?} pt: {pt:#?} ptb: {ptb:#?}");
        drop(ptb);
        println!("com_release call remove_instance_for_factory for {fuuid:#?}");
        CFPlugIn::remove_instance_for_factory(Some(&fuuid));
        println!("com_release drop fuuid: {fuuid:#?}");
        drop(fuuid);
        0
    }
}

extern "C-unwind" fn com_release2(this: *mut MetadataImporterPluginType) -> ULONG {
    let pt = &mut unsafe { *this };
    println!("com_release this: {this:#?} pt: {pt:#?}");
    if pt.refCount < 1 {
        panic!("ref count underflow");
    }
    pt.refCount = pt.refCount.checked_sub(1).unwrap();
    if pt.refCount != 0 {
        println!("com_release end (no-dealloc) this: {this:#?} pt: {pt:#?}");
        pt.refCount as ULONG
    } else {
        println!("com_release end (dealloc) this: {this:#?} pt: {pt:#?}");
        let fuuid = unsafe { CFRetained::from_raw(NonNull::new(pt.factoryID).unwrap()) };
        pt.factoryID = ptr::null_mut();
        let ptb = unsafe { Box::from_raw(this) };
        println!("com_release drop ptb: {this:#?} pt: {pt:#?} ptb: {ptb:#?}");
        drop(ptb);
        println!("com_release call remove_instance_for_factory for {fuuid:#?}");
        CFPlugIn::remove_instance_for_factory(Some(&fuuid));
        println!("com_release drop fuuid: {fuuid:#?}");
        drop(fuuid);
        0
    }
}

extern "C-unwind" fn com_importer_import_data(
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
    if true {
        let key = CFString::from_static_str(KMD_ITEM_DESCRIPTION);
        let val = CFString::from_static_str("this is GREAT");
        CFMutableDictionary::<CFString, CFString>::add(
            unsafe {
                attr.cast::<CFMutableDictionary<CFString, CFString>>()
                    .as_ref()
            }
            .unwrap(),
            &key,
            &val,
        );
    }

    false
}

static INTERFACE: MDImporterInterfaceStruct = MDImporterInterfaceStruct {
    _reserved: ptr::null_mut(),
    query_interface: Some(com_query_interface),
    add_ref: Some(com_add_ref),
    release: Some(com_release),
    importer_import_data: Some(com_importer_import_data),
};

#[unsafe(no_mangle)]
pub extern "C-unwind" fn MetadataImporterPluginFactory(
    allocator: *mut CFAllocator,
    inFactoryID: *mut CFUUID,
) -> *mut MetadataImporterPluginType {
    OsLogger::new("vin.je.minimal-importer")
        .level_filter(LevelFilter::Debug)
        .init()
        .unwrap();
    info!("MetadataImporterPluginFactory called");
    println!("passed allocator: {allocator:#?}");
    println!("passed uuid ptr: {inFactoryID:#?}");
    println!("");
    let uuid = unsafe { CFRetained::retain(NonNull::new(inFactoryID).unwrap()) };
    let importer_uuid = kMDImporterTypeID();
    println!("passed uuid: {uuid:#?} importer uuid: {importer_uuid:#?}");
    if uuid != importer_uuid {
        ptr::null_mut()
    } else {
        let ifu = MetadataImporterPluginFactoryUUID();
        CFPlugIn::add_instance_for_factory(Some(&ifu));
        let ifu_ptr = CFRetained::into_raw(ifu).as_ptr();
        let br = Box::new(MetadataImporterPluginType {
            conduitInterface: &INTERFACE,
            factoryID: ifu_ptr,
            refCount: 1,
        });
        let r = Box::into_raw(br);
        let dr = unsafe { r.as_ref() }.unwrap();
        println!("MetadataImporterPluginFactory returning r: {r:#?} dr: {dr:#?}");
        r
    }
}
