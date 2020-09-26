use std::sync::Arc;
use std::time::Duration;

use libnspire::{PID, PID_CX2, VID};
use rusb::GlobalContext;
use serde::{Deserialize, Serialize};

use crate::{Device, DeviceState};
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Promise {
  pub callback: String,
  pub error: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct DevId {
  pub bus_number: u8,
  pub address: u8,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  // your custom commands
  // multiple arguments are allowed
  // note that rename_all = "camelCase": you need to use "myCustomCommand" on JS
  Enumerate {
    #[serde(flatten)]
    promise: Promise,
  },
  #[serde(rename_all = "camelCase")]
  OpenDevice {
    #[serde(flatten)]
    promise: Promise,
    #[serde(flatten)]
    dev: DevId,
  },
  #[serde(rename_all = "camelCase")]
  CloseDevice {
    #[serde(flatten)]
    promise: Promise,
    #[serde(flatten)]
    dev: DevId,
  },
  #[serde(rename_all = "camelCase")]
  UpdateDevice {
    #[serde(flatten)]
    promise: Promise,
    #[serde(flatten)]
    dev: DevId,
  },
  #[serde(rename_all = "camelCase")]
  ListDir {
    #[serde(flatten)]
    promise: Promise,
    #[serde(flatten)]
    dev: DevId,
    path: String,
  },
  #[serde(rename_all = "camelCase")]
  DownloadFile {
    #[serde(flatten)]
    promise: Promise,
    #[serde(flatten)]
    dev: DevId,
    path: (String, u64),
    dest: String,
  },
  #[serde(rename_all = "camelCase")]
  UploadFile {
    #[serde(flatten)]
    promise: Promise,
    #[serde(flatten)]
    dev: DevId,
    path: String,
    src: String,
  },
  #[serde(rename_all = "camelCase")]
  UploadOs {
    #[serde(flatten)]
    promise: Promise,
    #[serde(flatten)]
    dev: DevId,
    src: String,
  },
  #[serde(rename_all = "camelCase")]
  DeleteFile {
    #[serde(flatten)]
    promise: Promise,
    #[serde(flatten)]
    dev: DevId,
    path: String,
  },
  #[serde(rename_all = "camelCase")]
  DeleteDir {
    #[serde(flatten)]
    promise: Promise,
    #[serde(flatten)]
    dev: DevId,
    path: String,
  },
  #[serde(rename_all = "camelCase")]
  CreateNspireDir {
    #[serde(flatten)]
    promise: Promise,
    #[serde(flatten)]
    dev: DevId,
    path: String,
  },
  #[serde(rename_all = "camelCase")]
  Move {
    #[serde(flatten)]
    promise: Promise,
    #[serde(flatten)]
    dev: DevId,
    src: String,
    dest: String,
  },
  #[serde(rename_all = "camelCase")]
  Copy {
    #[serde(flatten)]
    promise: Promise,
    #[serde(flatten)]
    dev: DevId,
    src: String,
    dest: String,
  },
  #[serde(rename_all = "camelCase")]
  SelectFile {
    #[serde(flatten)]
    promise: Promise,
    filter: Vec<String>,
  },
  #[serde(rename_all = "camelCase")]
  SelectFiles {
    #[serde(flatten)]
    promise: Promise,
    filter: Vec<String>,
  },
  #[serde(rename_all = "camelCase")]
  SelectFolder {
    #[serde(flatten)]
    promise: Promise,
  },
}

pub fn add_device(dev: Arc<rusb::Device<GlobalContext>>) -> rusb::Result<((u8, u8), Device)> {
  let descriptor = dev.device_descriptor()?;
  if !(descriptor.vendor_id() == VID && matches!(descriptor.product_id(), PID | PID_CX2)) {
    return Err(rusb::Error::Other);
  }
  let handle = dev.open()?;

  Ok((
    (dev.bus_number(), dev.address()),
    Device {
      name: handle.read_product_string(
        handle.read_languages(Duration::from_millis(100))?[0],
        &descriptor,
        Duration::from_millis(100),
      )?,
      device: dev,
      state: DeviceState::Closed,
    },
  ))
}

pub fn enumerate() -> Result<(), libnspire::Error> {
  crate::DEVICES.write().unwrap().extend(
    rusb::devices()?
      .iter()
      .filter_map(|dev| add_device(Arc::new(dev)).ok()),
  );
  Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDevice {
  #[serde(flatten)]
  pub dev: DevId,
  pub name: String,
  pub is_cx_ii: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressUpdate {
  #[serde(flatten)]
  pub dev: DevId,
  pub remaining: usize,
  pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
  pub path: String,
  pub is_dir: bool,
  pub date: u64,
  pub size: u64,
}
