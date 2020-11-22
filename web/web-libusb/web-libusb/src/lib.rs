#![feature(c_variadic)]
#![allow(non_camel_case_types)]
#![allow(improper_ctypes)]

use std::cell::RefCell;
use std::os::raw::{c_int, c_uint, c_void};
use std::ptr::null_mut;
use std::{mem, slice};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::DedicatedWorkerGlobalScope;

use crate::constants::*;
use crate::msgs::call;

use js_sys::Int32Array;

mod alloc;
pub mod constants;
mod misc;
mod msgs;
mod structs;

pub use libc::*;
pub use misc::*;
pub use structs::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! println {
    () => (
        $crate::log()
    );
    ($($arg:tt)*) => (
        $crate::log(&format!($($arg)*))
    )
}

#[macro_export]
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}]", file!(), line!());
    };
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    // Trailing comma with single argument is ignored
    ($val:expr,) => { dbg!($val) };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

thread_local! {
    static REPLY_BUF: RefCell<Vec<u8>> = RefCell::new(vec![0; 10000]);
}

fn global() -> DedicatedWorkerGlobalScope {
    js_sys::global().unchecked_into()
}

pub struct libusb_context {}

pub struct libusb_device_handle {
    device: *mut libusb_device,
}

impl libusb_device_handle {
    unsafe fn new(device: *mut libusb_device) -> *mut libusb_device_handle {
        libusb_ref_device(device);
        Box::into_raw(Box::new(libusb_device_handle { device }))
    }
}

pub struct libusb_device {
    id: u32,
    usb_id: (u16, u16),
    comm: Int32Array,
    refs: usize,
}

pub fn init(id: u32, usb_id: (u16, u16), comm: Int32Array) -> *mut libusb_device_handle {
    let device = Box::into_raw(Box::new(libusb_device {
        id,
        usb_id,
        comm,
        refs: 0,
    }));
    unsafe { libusb_device_handle::new(device) }
}

#[no_mangle]
pub unsafe extern "C" fn libusb_init(context: *mut *mut libusb_context) -> c_int {
    *context = Box::into_raw(Box::new(libusb_context {}));
    LIBUSB_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn libusb_exit(context: *mut libusb_context) {
    Box::from_raw(context);
}

#[no_mangle]
pub unsafe extern "C" fn libusb_close(dev_handle: *mut crate::libusb_device_handle) {
    libusb_unref_device((*dev_handle).device);
    Box::from_raw(dev_handle);
}

#[no_mangle]
pub unsafe extern "C" fn libusb_get_device_descriptor(
    dev: *const libusb_device,
    desc: *mut libusb_device_descriptor,
) -> c_int {
    *desc = libusb_device_descriptor {
        idVendor: (*dev).usb_id.0,
        idProduct: (*dev).usb_id.1,
        ..Default::default()
    };
    LIBUSB_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn libusb_ref_device(dev: *mut libusb_device) -> *mut libusb_device {
    (*dev).refs += 1;
    dev
}

#[no_mangle]
pub unsafe extern "C" fn libusb_unref_device(dev: *mut libusb_device) {
    (*dev).refs -= 1;
    if (*dev).refs == 0 {
        Box::from_raw(dev);
    }
}

#[no_mangle]
pub unsafe extern "C" fn libusb_get_device(
    dev_handle: *mut libusb_device_handle,
) -> *mut libusb_device {
    (*dev_handle).device
}

#[no_mangle]
pub unsafe extern "C" fn libusb_bulk_transfer(
    handle: *mut libusb_device_handle,
    endpoint: u8,
    data: *mut u8,
    length: c_int,
    transferred: *mut c_int,
    timeout: c_uint,
) -> c_int {
    let device = (*(*handle).device).id;
    if endpoint & LIBUSB_ENDPOINT_DIR_MASK == LIBUSB_ENDPOINT_OUT {
        match REPLY_BUF.with(|buf| {
            let mut buf = buf.borrow_mut();
            call(
                &(*(*handle).device).comm,
                msgs::BulkTransferOut {
                    device,
                    endpoint,
                    data: slice::from_raw_parts(data, length as usize),
                },
                &mut buf,
            )
        }) {
            Ok(reply) => match reply.0 {
                Ok(size) => {
                    if !transferred.is_null() {
                        *transferred = size as c_int;
                    }
                    LIBUSB_SUCCESS
                }
                Err(e) => e.into(),
            },
            Err(_) => LIBUSB_ERROR_OTHER,
        }
    } else {
        REPLY_BUF.with(|buf| {
            let mut buf = buf.borrow_mut();
            match call(
                &(*(*handle).device).comm,
                msgs::BulkTransferIn {
                    device,
                    endpoint,
                    length: length as usize,
                },
                &mut buf,
            ) {
                Ok(reply) => match reply.0 {
                    Ok(buf) => {
                        slice::from_raw_parts_mut(data, (buf.0).len().min(length as usize))
                            .copy_from_slice(buf.0);
                        if !transferred.is_null() {
                            *transferred = (buf.0).len() as c_int;
                        }
                        LIBUSB_SUCCESS
                    }
                    Err(e) => e.into(),
                },
                Err(_) => LIBUSB_ERROR_OTHER,
            }
        })
    }
}

#[no_mangle]
pub unsafe extern "C" fn libusb_set_configuration(
    handle: *mut libusb_device_handle,
    config: c_int,
) -> c_int {
    let device = (*(*handle).device).id;
    match REPLY_BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        call(
            &(*(*handle).device).comm,
            msgs::SelectConfiguration {
                device,
                config: config as u8,
            },
            &mut buf,
        )
    }) {
        Ok(reply) => match reply.0 {
            Ok(()) => LIBUSB_SUCCESS,
            Err(e) => e.into(),
        },
        Err(_) => LIBUSB_ERROR_OTHER,
    }
}

#[no_mangle]
pub unsafe extern "C" fn libusb_claim_interface(
    handle: *mut libusb_device_handle,
    interface_number: c_int,
) -> c_int {
    let device = (*(*handle).device).id;
    match REPLY_BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        call(
            &(*(*handle).device).comm,
            msgs::ClaimInterface {
                device,
                number: interface_number as u8,
            },
            &mut buf,
        )
    }) {
        Ok(reply) => match reply.0 {
            Ok(()) => LIBUSB_SUCCESS,
            Err(e) => e.into(),
        },
        Err(_) => LIBUSB_ERROR_OTHER,
    }
}

#[no_mangle]
pub unsafe extern "C" fn libusb_release_interface(
    handle: *mut libusb_device_handle,
    interface_number: c_int,
) -> c_int {
    let device = (*(*handle).device).id;
    match REPLY_BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        call(
            &(*(*handle).device).comm,
            msgs::ReleaseInterface {
                device,
                number: interface_number as u8,
            },
            &mut buf,
        )
    }) {
        Ok(reply) => match reply.0 {
            Ok(()) => LIBUSB_SUCCESS,
            Err(e) => e.into(),
        },
        Err(_) => LIBUSB_ERROR_OTHER,
    }
}

#[no_mangle]
pub unsafe extern "C" fn libusb_reset_device(handle: *mut libusb_device_handle) -> c_int {
    let device = (*(*handle).device).id;
    match REPLY_BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        call(
            &(*(*handle).device).comm,
            msgs::ResetDevice { device },
            &mut buf,
        )
    }) {
        Ok(reply) => match reply.0 {
            Ok(()) => LIBUSB_SUCCESS,
            Err(e) => e.into(),
        },
        Err(_) => LIBUSB_ERROR_OTHER,
    }
}

#[no_mangle]
pub unsafe extern "C" fn libusb_get_active_config_descriptor(
    handle: *const libusb_device,
    config_ptr: *mut *const libusb_config_descriptor,
) -> c_int {
    let device = (*handle).id;
    match REPLY_BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        call(
            &(*handle).comm,
            msgs::ActiveConfigDescriptor { device },
            &mut buf,
        )
    }) {
        Ok(reply) => match reply.0 {
            Ok(msg) => {
                let mut config: Box<libusb_config_descriptor> = Box::new(mem::zeroed());
                config.bConfigurationValue = msg.configuration_value;
                let interfaces: Vec<_> = msg
                    .interfaces
                    .into_iter()
                    .map(|interface| {
                        let settings: Vec<_> = interface
                            .into_iter()
                            .map(|setting| {
                                let endpoints: Vec<_> = setting
                                    .endpoints
                                    .into_iter()
                                    .map(|endpoint| libusb_endpoint_descriptor {
                                        bLength: 0,
                                        bDescriptorType: 0,
                                        bEndpointAddress: endpoint.address,
                                        bmAttributes: 0,
                                        wMaxPacketSize: endpoint.packet_size,
                                        bInterval: 0,
                                        bRefresh: 0,
                                        bSynchAddress: 0,
                                        extra: null_mut(),
                                        extra_length: 0,
                                    })
                                    .collect();
                                let endpoints = Box::into_raw(endpoints.into_boxed_slice());
                                libusb_interface_descriptor {
                                    bLength: 0,
                                    bDescriptorType: 0,
                                    bInterfaceNumber: 0,
                                    bAlternateSetting: setting.alternate_setting,
                                    bNumEndpoints: (*endpoints).len() as u8,
                                    bInterfaceClass: setting.interface_class,
                                    bInterfaceSubClass: setting.interface_subclass,
                                    bInterfaceProtocol: setting.interface_protocol,
                                    iInterface: 0,
                                    endpoint: (*endpoints).as_ptr(),
                                    extra: null_mut(),
                                    extra_length: 0,
                                }
                            })
                            .collect();
                        let settings = Box::into_raw(settings.into_boxed_slice());
                        libusb_interface {
                            altsetting: (*settings).as_ptr(),
                            num_altsetting: (*settings).len() as c_int,
                        }
                    })
                    .collect();
                let interfaces = Box::into_raw(interfaces.into_boxed_slice());
                config.interface = (*interfaces).as_ptr();
                config.bNumInterfaces = (*interfaces).len() as u8;
                *config_ptr = Box::into_raw(config);
                LIBUSB_SUCCESS
            }
            Err(e) => e.into(),
        },
        Err(_) => LIBUSB_ERROR_OTHER,
    }
}

#[no_mangle]
pub unsafe extern "C" fn libusb_free_config_descriptor(
    config_ptr: *const libusb_config_descriptor,
) {
    let config: Box<libusb_config_descriptor> = Box::from_raw(config_ptr as *mut _);
    let interfaces = Box::from_raw(&mut slice::from_raw_parts(
        config.interface,
        config.bNumInterfaces as usize,
    ));
    for interface in interfaces.iter() {
        let settings = Box::from_raw(&mut slice::from_raw_parts(
            interface.altsetting,
            interface.num_altsetting as usize,
        ));
        for setting in settings.iter() {
            Box::from_raw(&mut slice::from_raw_parts(
                setting.endpoint,
                setting.bNumEndpoints as usize,
            ));
        }
    }
}
