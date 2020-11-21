use crate::global;
use js_sys::{Atomics, Int32Array, Uint8Array};
use serde::{Deserialize, Serialize};
use std::os::raw::c_int;
use wasm_bindgen::prelude::*;
use std::fmt::Debug;
use crate::dbg;

#[derive(Debug, Deserialize)]
pub enum Error {
    NotFound,
    Security,
    Network,
    Abort,
    InvalidState,
    InvalidAccess,
    Unknown,
}

impl From<Error> for c_int {
    fn from(_: Error) -> Self {
        unimplemented!()
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "usbCmd", rename_all = "camelCase")]
pub enum Cmd<'a> {
    BulkTransferOut(BulkTransferOut<'a>),
    BulkTransferIn(BulkTransferIn),
    SelectConfiguration(SelectConfiguration),
    ClaimInterface(ClaimInterface),
    ReleaseInterface(ReleaseInterface),
    ResetDevice(ResetDevice),
    ActiveConfigDescriptor(ActiveConfigDescriptor),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NullReply(pub Result<(), Error>);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkTransferOut<'a> {
    pub device: u32,
    pub endpoint: u8,
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
}

impl<'a> From<BulkTransferOut<'a>> for Cmd<'a> {
    fn from(c: BulkTransferOut<'a>) -> Self {
        Cmd::BulkTransferOut(c)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkTransferOutReply(pub Result<usize, Error>);

impl<'reply, 'cmd> Message<'reply, 'cmd> for BulkTransferOut<'cmd> {
    type Reply = BulkTransferOutReply;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkTransferIn {
    pub device: u32,
    pub endpoint: u8,
    pub length: usize,
}

impl<'a> From<BulkTransferIn> for Cmd<'a> {
    fn from(c: BulkTransferIn) -> Self {
        Cmd::BulkTransferIn(c)
    }
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct Data<'a>(#[serde(with = "serde_bytes")] pub &'a [u8]);

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
#[serde(rename_all = "camelCase")]
pub struct BulkTransferInReply<'a>(pub Result<Data<'a>, Error>);

impl<'reply, 'cmd> Message<'reply, 'cmd> for BulkTransferIn {
    type Reply = BulkTransferInReply<'reply>;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectConfiguration {
    pub device: u32,
    pub config: u8,
}

impl<'a> From<SelectConfiguration> for Cmd<'a> {
    fn from(c: SelectConfiguration) -> Self {
        Cmd::SelectConfiguration(c)
    }
}

impl<'reply, 'cmd> Message<'reply, 'cmd> for SelectConfiguration {
    type Reply = NullReply;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimInterface {
    pub device: u32,
    pub number: u8,
}

impl<'a> From<ClaimInterface> for Cmd<'a> {
    fn from(c: ClaimInterface) -> Self {
        Cmd::ClaimInterface(c)
    }
}

impl<'reply, 'cmd> Message<'reply, 'cmd> for ClaimInterface {
    type Reply = NullReply;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseInterface {
    pub device: u32,
    pub number: u8,
}

impl<'a> From<ReleaseInterface> for Cmd<'a> {
    fn from(c: ReleaseInterface) -> Self {
        Cmd::ReleaseInterface(c)
    }
}

impl<'reply, 'cmd> Message<'reply, 'cmd> for ReleaseInterface {
    type Reply = NullReply;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetDevice {
    pub device: u32,
}

impl<'a> From<ResetDevice> for Cmd<'a> {
    fn from(c: ResetDevice) -> Self {
        Cmd::ResetDevice(c)
    }
}

impl<'reply, 'cmd> Message<'reply, 'cmd> for ResetDevice {
    type Reply = NullReply;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveConfigDescriptor {
    pub device: u32,
}

impl<'a> From<ActiveConfigDescriptor> for Cmd<'a> {
    fn from(c: ActiveConfigDescriptor) -> Self {
        Cmd::ActiveConfigDescriptor(c)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct USBEndpoint {
    pub address: u8,
    pub packet_size: u16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct USBAlternateInterface {
    pub alternate_setting: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub endpoints: Vec<USBEndpoint>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct USBConfiguration {
    pub configuration_value: u8,
    pub interfaces: Vec<Vec<USBAlternateInterface>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveConfigDescriptorReply(pub Result<USBConfiguration, Error>);

impl<'reply, 'cmd> Message<'reply, 'cmd> for ActiveConfigDescriptor {
    type Reply = ActiveConfigDescriptorReply;
}

pub trait Message<'reply, 'cmd>: Into<Cmd<'cmd>> + Serialize {
    type Reply: Deserialize<'reply> + Debug;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub fn call<'reply, 'cmd, T: Message<'reply, 'cmd>>(
    arr: &Int32Array,
    msg: T,
    buf: &'reply mut [u8],
) -> Result<T::Reply, rmp_serde::decode::Error> {
    let value = serde_wasm_bindgen::to_value(&Into::<Cmd>::into(msg)).unwrap();
    let _ = global().post_message(&value);
    let _ = Atomics::wait(arr, 0, 0);
    let len = arr.get_index(0) as u32;
    let _ = Atomics::store(arr, 0, 0);
    let js_buf = Uint8Array::new(&arr.buffer()).subarray(4, 4 + len);
    let buf = &mut buf[0..(js_buf.length() as usize)];
    js_buf.copy_to(buf);
    match rmp_serde::from_read_ref(&buf[..]) {
        Ok(v) => Ok(v),
        Err(e) => Err({
            log(&format!("{}", e));
            e
        }),
    }
}
