#![deny(warnings)]

use std::mem::MaybeUninit;
use std::ffi::{CString, CStr};
use std::marker::PhantomData;

pub mod ffi;

pub struct Nfc {
    context: *mut ffi::context_t,
}

impl Nfc {
    pub fn new() -> Option<Self> {

        let mut context_uninit = MaybeUninit::<*mut ffi::context_t>::uninit();
        let context = unsafe {
            ffi::nfc_init(context_uninit.as_mut_ptr());
            if context_uninit.as_mut_ptr() == std::ptr::null_mut() {
                return None;
            }
            context_uninit.assume_init()
        };

        Some(Nfc { context })
    }

    pub fn gatekeeper_device(&mut self) -> Option<NfcDevice> {
        let device_string = CString::new("pn532_uart:/dev/ttyUSB0").unwrap();
        let device = unsafe {
            let device_ptr = ffi::nfc_open(self.context, device_string.as_ptr());
            if device_ptr == std::ptr::null_mut() {
                return None;
            }
            device_ptr
        };
        Some(NfcDevice { device, _context_lifetime: PhantomData })
    }
}

impl Drop for Nfc {
    fn drop(&mut self) {
        unsafe {
            ffi::nfc_exit(self.context);
        }
    }
}

pub struct NfcDevice<'a> {
    device: *mut ffi::device_t,
    _context_lifetime: std::marker::PhantomData<&'a ()>,
}

impl NfcDevice<'_> {
    pub fn first_tag(&mut self) -> Option<NfcTag> {

        let (tags, tag) = unsafe {
            let tags = ffi::freefare_get_tags(self.device);
            if tags == std::ptr::null_mut() { return None; }

            let tag = *tags;
            if tag == std::ptr::null_mut() { return None; }
            (tags, tag)
        };

        Some(NfcTag { tags, tag, _device_lifetime: PhantomData })
    }
}

impl Drop for NfcDevice<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::nfc_close(self.device);
        }
    }
}

pub struct NfcTag <'a> {
    tags: *mut *mut ffi::mifare_t,
    tag: *mut ffi::mifare_t,
    _device_lifetime: std::marker::PhantomData<&'a ()>,
}

impl NfcTag<'_> {
    pub fn get_uid(&mut self) -> Option<String> {
        unsafe {
            let tag_uid = ffi::freefare_get_tag_uid(self.tag);
            if tag_uid == std::ptr::null_mut() { return None; }
            let tag_uid_string = CString::from_raw(tag_uid);
            Some(tag_uid_string.to_string_lossy().to_string())
        }
    }

    pub fn get_friendly_name(&mut self) -> Option<&str> {
        unsafe {
            let tag_name = ffi::freefare_get_tag_friendly_name(self.tag);
            let tag_name_string = CStr::from_ptr(tag_name);
            tag_name_string.to_str().ok()
        }
    }
}

impl Drop for NfcTag<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::freefare_free_tags(self.tags);
        }
    }
}
