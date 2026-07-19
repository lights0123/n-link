use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use libnspire::{PID, PID_CX2, VID};
use rusb::GlobalContext;
use serde::{Deserialize, Serialize};

pub enum DeviceState {
  Open(
    Arc<Mutex<libnspire::Handle<GlobalContext>>>,
    libnspire::info::Info,
  ),
  Closed,
}

pub struct Device {
  pub name: String,
  pub device: Arc<rusb::Device<GlobalContext>>,
  pub state: DeviceState,
  pub needs_drivers: bool,
}

lazy_static::lazy_static! {
  pub static ref DEVICES: RwLock<HashMap<(u8, u8), Device>> = RwLock::new(HashMap::new());
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct DevId {
  pub bus_number: u8,
  pub address: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDevice {
  #[serde(flatten)]
  pub dev: DevId,
  pub name: String,
  pub is_cx_ii: bool,
  pub needs_drivers: bool,
}

#[derive(Debug, Clone, Serialize)]
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

pub fn get_open_dev(
  dev: &DevId,
) -> Result<Arc<Mutex<libnspire::Handle<GlobalContext>>>, anyhow::Error> {
  if let Some(dev) = DEVICES.read().unwrap().get(&(dev.bus_number, dev.address)) {
    match &dev.state {
      DeviceState::Open(handle, _) => Ok(handle.clone()),
      DeviceState::Closed => anyhow::bail!("Device closed"),
    }
  } else {
    anyhow::bail!("Failed to find device");
  }
}
