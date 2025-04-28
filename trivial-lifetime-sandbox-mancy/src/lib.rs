use std::ffi::c_void;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};

/// 128-bit GUID / IID compatible with COM and CFUUID
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct IID(pub [u8; 16]);

/// Trait for all COM-style interfaces.
pub unsafe trait ComInterface: ComVtbl {
    const IID: IID;
}

/// Base trait for exposing a vtable and type-erased QueryInterface.
pub unsafe trait ComVtbl {
    type VTable: 'static;

    fn vtbl(&self) -> &Self::VTable;
    fn as_raw(&self) -> *mut c_void;

    /// Polymorphic cast to a different interface via runtime IID match.
    fn query_interface<U: ComInterface>(&self) -> Option<ComPtr<U>>;
}

/// Base layout for vtables with QueryInterface, AddRef, Release
#[repr(C)]
#[derive(Debug)]
pub struct VTableBase {
    pub query_interface:
        Option<unsafe extern "C" fn(*mut c_void, iid: *const IID, out: *mut *mut c_void) -> i32>,
    pub add_ref: Option<unsafe extern "C" fn(*mut c_void) -> u32>,
    pub release: Option<unsafe extern "C" fn(*mut c_void) -> u32>,
}

/// Reference-counted smart pointer for COM-style interfaces.
pub struct ComPtr<T: ComInterface> {
    ptr: *mut T,
    _marker: PhantomData<T>,
}

impl<T: ComInterface> ComPtr<T> {
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn query_interface<U: ComInterface>(&self) -> Option<ComPtr<U>> {
        unsafe { (&*self.ptr).query_interface::<U>() }
    }
}

impl<T: ComInterface> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        unsafe {
            let base = *(self.ptr as *mut *const VTableBase);
            if let Some(add_ref) = (&*base).add_ref {
                add_ref(self.ptr as *mut _);
            }
        }
        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: ComInterface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            let base = *(self.ptr as *mut *const VTableBase);
            println!("base: {base:#?}");
            let a = &*base;
            println!("a: {a:#?}");
            // let b = &a;
            if let Some(release) = a.release {
                release(self.ptr as *mut _);
            }
        }
    }
}

impl<T: ComInterface> Deref for ComPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T: ComInterface> DerefMut for ComPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

/// Example COM interface
#[repr(C)]
pub struct IExampleVTable {
    pub base: VTableBase,
    pub get_value: Option<unsafe extern "C" fn(this: *mut c_void) -> i32>,
}

#[repr(C)]
pub struct ExampleImpl {
    pub vtbl: *const IExampleVTable,
    pub ref_cnt: AtomicU32,
    pub value: i32,
}

pub const IID_IEXAMPLE: IID = IID([
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe,
]);

unsafe extern "C" fn example_query_interface(
    this: *mut c_void,
    iid: *const IID,
    out: *mut *mut c_void,
) -> i32 {
    let this = this as *mut ExampleImpl;
    if *unsafe { &*iid } == IID_IEXAMPLE {
        unsafe {
            *out = this as *mut _;
            example_add_ref(this as *mut _);
        }
        0
    } else {
        unsafe {
            *out = ptr::null_mut();
        }
        1
    }
}

unsafe extern "C" fn example_add_ref(this: *mut c_void) -> u32 {
    let this = this as *mut ExampleImpl;
    (unsafe { &*this }).ref_cnt.fetch_add(1, Ordering::Relaxed) + 1
}

unsafe extern "C" fn example_release(this: *mut c_void) -> u32 {
    let this = this as *mut ExampleImpl;
    let rc = (unsafe { &*this }).ref_cnt.fetch_sub(1, Ordering::Release) - 1;
    if rc == 0 {
        std::sync::atomic::fence(Ordering::Acquire);
        drop(unsafe { Box::from_raw(this) });
    }
    rc
}

unsafe extern "C" fn example_get_value(this: *mut c_void) -> i32 {
    (unsafe { &*(this as *mut ExampleImpl) }).value
}

static IEXAMPLE_VTBL: IExampleVTable = IExampleVTable {
    base: VTableBase {
        query_interface: Some(example_query_interface),
        add_ref: Some(example_add_ref),
        release: Some(example_release),
    },
    get_value: Some(example_get_value),
};

unsafe impl ComVtbl for ExampleImpl {
    type VTable = IExampleVTable;
    fn vtbl(&self) -> &Self::VTable {
        unsafe { &*self.vtbl }
    }
    fn as_raw(&self) -> *mut c_void {
        self as *const _ as *mut _
    }
    fn query_interface<U: ComInterface>(&self) -> Option<ComPtr<U>> {
        if U::IID == IID_IEXAMPLE {
            Some(unsafe { ComPtr::from_raw(self as *const _ as *mut U) })
        } else {
            None
        }
    }
}

unsafe impl ComInterface for ExampleImpl {
    const IID: IID = IID_IEXAMPLE;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_com() {
        let boxed = Box::new(ExampleImpl {
            vtbl: &IEXAMPLE_VTBL,
            ref_cnt: AtomicU32::new(1),
            value: 1337,
        });
        let ptr = unsafe { ComPtr::from_raw(Box::into_raw(boxed)) };
        let val = unsafe { ((&*ptr).vtbl().get_value.unwrap())(ptr.as_raw()) };
        println!("Value: {val}");
        let q: Option<ComPtr<ExampleImpl>> = ptr.query_interface();
        assert!(q.is_some());
    }
}
