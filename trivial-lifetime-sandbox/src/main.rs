use std::ffi::c_void;
use std::ptr;

#[repr(C)]
#[derive(Debug)]
pub struct VTable {
    _reserved: *mut c_void,
    add_ref: Option<extern "C-unwind" fn(this: *mut Plugin) -> u32>,
    release: Option<extern "C-unwind" fn(this: *mut Plugin) -> u32>,
    get_refcnt: Option<extern "C-unwind" fn(this: *const Plugin) -> u32>,
}

// *mut c_void sharing between threads
// whos afraid of a bit of UB? ( ͡° ͜ʖ ͡°)
unsafe impl Sync for VTable {}
// apparently Send isn't needed?
// unsafe impl Send for VTable {}

impl AsRef<VTable> for VTable {
    fn as_ref(&self) -> &VTable {
        &self
    }
}

impl AsMut<VTable> for VTable {
    fn as_mut(&mut self) -> &mut VTable {
        self
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Plugin {
    vtbl_ptr: *const VTable,
    ref_cnt: u32,
}

impl Plugin {
    // pub fn as_ptr(&mut self) -> *mut Plugin {
    //     self
    // }
    pub fn intf(&self) -> &VTable {
        unsafe { self.vtbl_ptr.as_ref() }.unwrap()
    }
    pub fn add_ref(&mut self) -> u32 {
        println!("Plugin::add_ref self: {self:#?}");
        self.ref_cnt.checked_add(1).unwrap()
    }
    pub fn release(&mut self) -> u32 {
        println!("Plugin::release self: {self:#?}");
        if self.ref_cnt < 1 {
            panic!("ref count underflow");
        }
        let rc = self.ref_cnt.checked_sub(1).unwrap();
        if rc != 0 {
            rc
        } else {
            println!("Plugin::release RC zeroed, should free here");
            rc
        }
    }
    pub fn get_refcnt(&self) -> u32 {
        println!("Plugin::get_refcnt self: {self:#?}");
        self.ref_cnt
    }
}

extern "C-unwind" fn com_add_ref(this: *mut Plugin) -> u32 {
    println!("com_add_ref this: {this:#?}");
    let hndl = unsafe { this.as_mut() }.unwrap();
    println!("com_add_ref this: {this:#?} hndl: {hndl:#?}");
    hndl.add_ref()
}

extern "C-unwind" fn com_release(this: *mut Plugin) -> u32 {
    let hndl = unsafe { this.as_mut() }.unwrap();
    println!("com_release this: {this:#?} hndl: {hndl:#?}");
    hndl.release()
}

extern "C-unwind" fn com_get_refcnt(this: *const Plugin) -> u32 {
    println!("com_get_refcnt this: {this:#?}");
    let hndl = unsafe { this.as_ref() }.unwrap();
    println!("com_get_refcnt this: {this:#?} hndl: {hndl:#?}");
    hndl.get_refcnt()
}

static INTERFACE: VTable = VTable {
    _reserved: ptr::null_mut(),
    add_ref: Some(com_add_ref),
    release: Some(com_release),
    get_refcnt: Some(com_get_refcnt),
};

#[unsafe(no_mangle)]
pub extern "C-unwind" fn PluginFactory(do_fail: bool) -> *mut Plugin {
    println!("PluginFactory called");
    if do_fail {
        println!("PluginFactory asked to return null");
        ptr::null_mut()
    } else {
        let br = Box::new(Plugin {
            vtbl_ptr: &INTERFACE,
            ref_cnt: 1,
        });
        let r = Box::into_raw(br);
        let dr = unsafe { r.as_ref() }.unwrap();
        println!("PluginFactory returning r: {r:#?} dr: {dr:#?}");
        r
    }
}

fn main() {
    println!("trivial-lifetime-sandbox-begin");
    let failed_plugin = PluginFactory(true);
    if failed_plugin.is_null() {
        println!("failed_plugin returned null as expected");
    } else {
        let failed_plugin_opt = unsafe { failed_plugin.as_ref() };
        match failed_plugin_opt {
            None => {
                println!("failed_plugin_opt was None")
            }
            Some(x) => {
                println!("failed_plugin_opt is: {x:#?}")
            }
        }
        println!(
            "failed_plugin returned non-null unexpectedly: {failed_plugin:#?} {failed_plugin_opt:#?}"
        );
    }
    let good_plugin = PluginFactory(false);
    if good_plugin.is_null() {
        println!("good_plugin returned null unexpectedly");
    } else {
        let good_plugin_opt = unsafe { good_plugin.as_mut() };
        match good_plugin_opt {
            None => {
                println!("good_plugin_opt was None")
            }
            Some(p) => {
                println!("good_plugin_opt is: {p:#?}");
                let rc0 = p.get_refcnt();
                println!("rc0: {rc0} p: {p:#?}");
                let rc_inc0 = p.add_ref();
                let rc1 = p.get_refcnt();
                println!("rc_inc0: {rc_inc0} rc1: {rc1} p: {p:#?}");
                let rc_inc1 = p.add_ref();
                let rc2 = p.get_refcnt();
                println!("rc_inc1: {rc_inc1} rc2: {rc2} p: {p:#?}");
                let rc_dec1 = p.release();
                let rc3 = p.get_refcnt();
                println!("rc_dec1: {rc_dec1} rc3: {rc3} p: {p:#?}");
                let rc_dec0 = p.release();
                let rc4 = p.get_refcnt();
                println!("rc_dec0: {rc_dec0} rc4: {rc4} p: {p:#?}");
                let rc_decn1 = p.release();
                println!("rc_decn1: {rc_decn1} p: {p:#?}");
                let rc5 = p.get_refcnt();
                println!("rc5: {rc5} p: {p:#?}");
                let rc_decn2 = p.release();
                println!("rc_decn2: {rc_decn2} p: {p:#?}");
                let rc6 = p.get_refcnt();
                println!("rc6: {rc6} p: {p:#?}");
            }
        }
        println!("good_plugin returned non-null as expected: {good_plugin:#?}");
    }
}
