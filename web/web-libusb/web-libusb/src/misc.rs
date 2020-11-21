use std::os::raw::{c_char, c_int, c_uchar, c_uint, c_void};
use std::ptr::null_mut;

pub type libusb_hotplug_callback_handle = c_int;
pub type libusb_hotplug_callback_fn = extern "system" fn(
    ctx: *mut crate::libusb_context,
    device: *mut crate::libusb_device,
    event: crate::libusb_hotplug_event,
    user_data: *mut c_void,
) -> c_int;
#[repr(C)]
pub struct libusb_version {
    pub major: u16,
    pub minor: u16,
    pub micro: u16,
    pub nano: u16,
    pub rc: *const c_char,
    pub describe: *const c_char,
}

unsafe impl Send for libusb_version {}
unsafe impl Sync for libusb_version {}

#[no_mangle]
pub unsafe extern "C" fn libusb_get_version() -> *const libusb_version {
    static LIBUSB_VERSION: libusb_version = libusb_version {
        major: 0,
        minor: 0,
        micro: 0,
        nano: 0,
        rc: b"\0".as_ptr() as _,
        describe: b"\0".as_ptr() as _,
    };
    &LIBUSB_VERSION
}
#[no_mangle]
pub unsafe extern "C" fn libusb_has_capability(capability: u32) -> c_int {
    match capability {
        crate::constants::LIBUSB_CAP_HAS_CAPABILITY => 1,
        _ => 0,
    }
}

extern "C" {
    pub fn libusb_set_option(ctx: *mut crate::libusb_context, option: u32, ...) -> c_int;
    pub fn libusb_open(
        dev: *const crate::libusb_device,
        handle: *mut *mut crate::libusb_device_handle,
    ) -> c_int;
    pub fn libusb_open_device_with_vid_pid(
        context: *mut crate::libusb_context,
        vendor_id: u16,
        product_id: u16,
    ) -> *mut crate::libusb_device_handle;
    pub fn libusb_set_debug(context: *mut crate::libusb_context, level: c_int);
    pub fn libusb_hotplug_register_callback(
        ctx: *mut crate::libusb_context,
        events: crate::libusb_hotplug_event,
        flags: crate::libusb_hotplug_flag,
        vendor_id: c_int,
        product_id: c_int,
        dev_class: c_int,
        cb_fn: libusb_hotplug_callback_fn,
        user_data: *mut crate::c_void,
        callback_handle: *mut libusb_hotplug_callback_handle,
    ) -> c_int;
    pub fn libusb_hotplug_deregister_callback(
        ctx: *mut crate::libusb_context,
        callback_handle: libusb_hotplug_callback_handle,
    );
    pub fn libusb_handle_events_timeout_completed(
        context: *mut crate::libusb_context,
        tv: *const libc::timeval,
        completed: *mut c_int,
    ) -> c_int;
    pub fn libusb_handle_events_completed(
        context: *mut crate::libusb_context,
        completed: *mut c_int,
    ) -> c_int;
    pub fn libusb_get_device_descriptor(
        dev: *const crate::libusb_device,
        desc: *mut crate::libusb_device_descriptor,
    ) -> c_int;
    pub fn libusb_get_config_descriptor(
        dev: *const crate::libusb_device,
        index: u8,
        config: *mut *const crate::libusb_config_descriptor,
    ) -> c_int;
    pub fn libusb_get_bus_number(dev: *const crate::libusb_device) -> u8;
    pub fn libusb_get_port_number(dev: *mut crate::libusb_device) -> u8;
    pub fn libusb_get_port_numbers(
        dev: *mut crate::libusb_device,
        port_numbers: *mut u8,
        port_numbers_len: c_int,
    ) -> c_int;
    pub fn libusb_get_device_address(dev: *const crate::libusb_device) -> u8;
    pub fn libusb_get_device_speed(dev: *const crate::libusb_device) -> c_int;
    pub fn libusb_get_max_packet_size(dev: *const crate::libusb_device, endpoint: c_uchar)
        -> c_int;
    pub fn libusb_get_max_iso_packet_size(
        dev: *const crate::libusb_device,
        endpoint: c_uchar,
    ) -> c_int;
    pub fn libusb_get_configuration(
        dev_handle: *mut crate::libusb_device_handle,
        config: *mut c_int,
    ) -> c_int;
    pub fn libusb_interrupt_transfer(
        dev_handle: *mut crate::libusb_device_handle,
        endpoint: c_uchar,
        data: *mut c_uchar,
        length: c_int,
        transferred: *mut c_int,
        timeout: c_uint,
    ) -> c_int;
    pub fn libusb_control_transfer(
        dev_handle: *mut crate::libusb_device_handle,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: *mut c_uchar,
        length: u16,
        timeout: c_uint,
    ) -> c_int;
    pub fn libusb_clear_halt(
        dev_handle: *mut crate::libusb_device_handle,
        endpoint: c_uchar,
    ) -> c_int;
    pub fn libusb_set_auto_detach_kernel_driver(
        dev_handle: *mut crate::libusb_device_handle,
        enable: c_int,
    ) -> c_int;
    pub fn libusb_kernel_driver_active(
        dev_handle: *mut crate::libusb_device_handle,
        interface_number: c_int,
    ) -> c_int;
    pub fn libusb_detach_kernel_driver(
        dev_handle: *mut crate::libusb_device_handle,
        interface_number: c_int,
    ) -> c_int;
    pub fn libusb_attach_kernel_driver(
        dev_handle: *mut crate::libusb_device_handle,
        interface_number: c_int,
    ) -> c_int;
    pub fn libusb_set_interface_alt_setting(
        dev_handle: *mut crate::libusb_device_handle,
        interface_number: c_int,
        alternate_setting: c_int,
    ) -> c_int;
    pub fn libusb_get_string_descriptor_ascii(
        dev_handle: *mut crate::libusb_device_handle,
        desc_index: u8,
        data: *mut c_uchar,
        length: c_int,
    ) -> c_int;
    pub fn libusb_get_device_list(
        context: *mut crate::libusb_context,
        list: *mut *const *mut crate::libusb_device,
    ) -> isize;
    pub fn libusb_free_device_list(list: *const *mut crate::libusb_device, unref_devices: c_int);
}

#[no_mangle]
pub unsafe extern "C" fn memchr(mut ptr: *const c_void, value: c_int, num: usize) -> *mut c_void {
    for _ in 0..num {
        if *(ptr as *const u8) == value as u8 {
            return ptr as *mut c_void;
        }
        ptr = ptr.add(1);
    }
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn strncpy(
    mut dst: *mut c_char,
    mut src: *const c_char,
    n: usize,
) -> *mut c_char {
    let mut i = 0;
    while i < n && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    while i < n {
        *dst.add(i) = 0;
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strlen(mut cs: *const c_char) -> usize {
    let mut i = 0;
    while *cs != 0 {
        i += 1;
        cs = cs.add(1);
    }
    i
}
