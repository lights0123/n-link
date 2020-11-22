#![feature(c_variadic)]

use core::ffi::VaList;
use std::os::raw::*;
use std::ffi::CStr;
use libusb1_sys::{dbg, println};
use std::fmt;

#[derive(Debug)]
pub enum DoubleFormat {
    Normal,
    Standard,
    Auto,
}

#[derive(Debug)]
pub enum Argument<'a> {
    Extra(&'a [u8]),
    Percent,
    Int(c_int),
    Uint(c_uint),
    Double {
        value: c_uint,
        upper: bool,
        format: DoubleFormat,
    },
    String(&'a CStr),
    Char(u8),
    Hex(u8, c_int),
    Pointer(*const ()),
}

pub fn to_write<'a>(w: &'a mut impl fmt::Write) -> impl FnMut(Argument) + 'a {
    move |arg| match arg {
        Argument::Extra(d) => {w.write_str(&String::from_utf8_lossy(d));},
        Argument::Hex(len, data) => {write!(w, "{:#0width$x}", data, width=len as usize);},
        Argument::Int(data) => {write!(w, "{}", data);},
        e => {dbg!(e);},
    }
}

pub unsafe fn func(format: *const c_char, mut args: VaList, mut handler: impl FnMut(Argument)) -> c_int {
    let str = CStr::from_ptr(format).to_bytes();
    let mut iter = str.split(|&c| c == b'%');
    if let Some(begin) = iter.next() {
        handler(Argument::Extra(begin));
    }
    for mut sub in iter {
        let prefix = if let Some(init) = sub.get(..2) {
            std::str::from_utf8(init).ok().and_then(|s|s.parse().ok()).map(|a| {
                sub = &sub[2..];
                a
            })
        } else {
            None
        };
        let ch = sub.get(0).unwrap_or(&0);
        handler(match ch {
            b'%' => {
                Argument::Percent
            }
            b'd' | b'i' => {
                Argument::Int(args.arg())
            }
            b'x' => {
                Argument::Hex(prefix.unwrap_or(0), args.arg())
            }
            b'u' => {
                Argument::Uint(args.arg())
            }
            b'f' | b'F' => {
                Argument::Double {
                    value: args.arg(),
                    upper: ch.is_ascii_uppercase(),
                    format: DoubleFormat::Normal,
                }
            }
            b'e' | b'E' => {
                Argument::Double {
                    value: args.arg(),
                    upper: ch.is_ascii_uppercase(),
                    format: DoubleFormat::Standard,
                }
            }
            b'g' | b'G' => {
                Argument::Double {
                    value: args.arg(),
                    upper: ch.is_ascii_uppercase(),
                    format: DoubleFormat::Auto,
                }
            }
            b's' => {
                Argument::String(CStr::from_ptr(args.arg()))
            }
            b'c' => {
                Argument::Char(args.arg())
            }
            b'p' => {
                Argument::Pointer(args.arg())
            }
            x => {
                dbg!("unrec {}", x);
                return -1;
            }
        });
        handler(Argument::Extra(sub.get(1..).unwrap_or(&[])));
    }
    0
}
