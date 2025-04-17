use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use objc2::rc::Retained;
use objc2_foundation::{NSString, ns_string};

fn cf_string_to_ns(s: &CFString) -> &NSString {
    let ptr: CFStringRef = s.as_concrete_TypeRef();
    let ptr: *const NSString = ptr.cast();
    // SAFETY: CFString is toll-free bridged with NSString.
    unsafe { ptr.as_ref().unwrap() }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
