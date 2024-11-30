use crate::{SoundTTs, SoundValue};
use libc::c_char;
use std::ffi::{CStr, CString};


#[repr(C)]
pub struct DeviceList {
    ptr: *mut *const c_char,
    len: usize,
}


#[no_mangle]
pub extern "C" fn sound_tts_init() {
    SoundTTs::init();
}

#[no_mangle]
pub extern "C" fn sound_tts_get_devices() -> *mut DeviceList {
    let devices = SoundTTs::get_devices();
    let mut c_strings: Vec<*const c_char> = Vec::new();
    devices.iter().for_each(|device| {
        let c_string = CString::new(device.as_str()).expect("CString::new failed");
        let c = c_string.into_raw();
        c_strings.push(c);
    });
    let list = DeviceList {
        ptr: c_strings.as_mut_ptr(),
        len: c_strings.len(),
    };
    std::mem::forget(c_strings);
    Box::into_raw(Box::new(list))
}

#[no_mangle]
pub extern "C" fn sound_tts_free_device_list(list: *mut DeviceList) {
    if !list.is_null() {
        unsafe {
            let list = Box::from_raw(list);
            for i in 0..list.len {
                let _ = CString::from_raw(*list.ptr.add(i) as *mut c_char);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn sound_tts_device_is_exist(str: *const c_char) -> bool {
    let str = unsafe { CStr::from_ptr(str).to_string_lossy().into_owned() };
    SoundTTs::device_is_exist(str.as_str())
}


#[no_mangle]
pub extern "C" fn sound_tts_speak(str: *const c_char, play_count: u64, play_interval: u64, device_name: *const c_char, interrupt: bool) {
    if str.is_null() || device_name.is_null() {
        println!("Received a null pointer!");
        return;
    }
    let str = unsafe {
        let c_str = CStr::from_ptr(str);
        match c_str.to_str() {
            Ok(str) => {
                str
            }
            Err(_) => {
                ""
            }
        }
    };
    let device_name = unsafe { CStr::from_ptr(device_name).to_string_lossy().into_owned() };
    let value = SoundValue::new(str, play_count, play_interval, &device_name);
    SoundTTs::execute(value, interrupt);
}

#[no_mangle]
pub extern "C" fn sound_tts_stop(device: *const c_char) {
    let device_name = unsafe { CStr::from_ptr(device).to_string_lossy().into_owned() };
    SoundTTs::stop(&device_name);
}
