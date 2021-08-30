use std::sync::Arc;
use std::time::Duration;

use libnspire::{PID, PID_CX2, VID};
use rusb::GlobalContext;
use serde::{Deserialize, Serialize};
use tauri::{Runtime, Window};

use crate::{Device, DeviceState, SerializedError};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct DevId {
  pub bus_number: u8,
  pub address: u8,
}

pub fn add_device(dev: Arc<rusb::Device<GlobalContext>>) -> rusb::Result<((u8, u8), Device)> {
  let descriptor = dev.device_descriptor()?;
  if !(descriptor.vendor_id() == VID && matches!(descriptor.product_id(), PID | PID_CX2)) {
    return Err(rusb::Error::Other);
  }

  let (name, needs_drivers) = match dev.open() {
    Ok(handle) => (
      handle.read_product_string(
        handle.read_languages(Duration::from_millis(100))?[0],
        &descriptor,
        Duration::from_millis(100),
      )?,
      false,
    ),
    Err(rusb::Error::NotSupported) | Err(rusb::Error::Access) => (
      if descriptor.product_id() == PID_CX2 {
        "TI-Nspire CX II"
      } else {
        "TI-Nspire"
      }
      .to_string(),
      true,
    ),
    Err(other) => return Err(other),
  };

  Ok((
    (dev.bus_number(), dev.address()),
    Device {
      name,
      device: dev,
      state: DeviceState::Closed,
      needs_drivers,
    },
  ))
}

#[tauri::command]
pub fn enumerate<R: Runtime>(handle: Window<R>) -> Result<Vec<AddDevice>, SerializedError> {
  let devices: Vec<_> = rusb::devices()?.iter().collect();
  let mut map = crate::DEVICES.write().unwrap();
  map
    .drain_filter(|k, _v| {
      devices
        .iter()
        .all(|d| d.bus_number() != k.0 || d.address() != k.1)
    })
    .for_each(|d| {
      if let Err(msg) = handle.emit(
        "removeDevice",
        DevId {
          bus_number: (d.0).0,
          address: (d.0).1,
        },
      ) {
        eprintln!("{}", msg);
      }
    });
  let filtered: Vec<_> = devices
    .into_iter()
    .filter(|d| !map.contains_key(&(d.bus_number(), d.address())))
    .collect();
  Ok(
    filtered
      .into_iter()
      .filter_map(|dev| add_device(Arc::new(dev)).ok())
      .map(|dev| {
        let msg = AddDevice {
          dev: DevId {
            bus_number: (dev.0).0,
            address: (dev.0).1,
          },
          name: (dev.1).name.clone(),
          is_cx_ii: (dev.1)
            .device
            .device_descriptor()
            .map(|d| d.product_id() == PID_CX2)
            .unwrap_or(false),
          needs_drivers: (dev.1).needs_drivers,
        };
        map.insert(dev.0, dev.1);
        msg
      })
      .collect(),
  )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDevice {
  #[serde(flatten)]
  pub dev: DevId,
  pub name: String,
  pub is_cx_ii: bool,
  pub needs_drivers: bool,
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
