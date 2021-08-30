#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use hashbrown::HashMap;
use libnspire::{PID_CX2, VID};
use rusb::{GlobalContext, Hotplug, UsbContext};
use serde::Serialize;
use tauri::{Runtime, Window};

use crate::cmd::{add_device, AddDevice, DevId, ProgressUpdate};

mod cli;
mod cmd;

pub enum DeviceState {
  Open(
    Arc<Mutex<libnspire::Handle<GlobalContext>>>,
    libnspire::info::Info,
  ),
  Closed,
}

pub struct Device {
  name: String,
  device: Arc<rusb::Device<GlobalContext>>,
  state: DeviceState,
  needs_drivers: bool,
}
lazy_static::lazy_static! {
  static ref DEVICES: RwLock<HashMap<(u8, u8), Device>> = RwLock::new(HashMap::new());
}
struct DeviceMon<R: Runtime> {
  window: Window<R>,
}

impl<R: Runtime> Hotplug<GlobalContext> for DeviceMon<R> {
  fn device_arrived(&mut self, device: rusb::Device<GlobalContext>) {
    let handle = self.window.clone();
    let is_cx_ii = device
      .device_descriptor()
      .map(|d| d.product_id() == PID_CX2)
      .unwrap_or(false);
    let device = Arc::new(device);
    std::thread::spawn(move || loop {
      match add_device(device.clone()) {
        Ok(dev) => {
          let name = (dev.1).name.clone();
          let needs_drivers = (dev.1).needs_drivers;
          DEVICES.write().unwrap().insert(dev.0, dev.1);
          if let Err(msg) = handle.emit(
            "addDevice",
            AddDevice {
              dev: DevId {
                bus_number: (dev.0).0,
                address: (dev.0).1,
              },
              name,
              is_cx_ii,
              needs_drivers,
            },
          ) {
            eprintln!("{}", msg);
          };
          return;
        }
        Err(rusb::Error::Busy) => {
          println!("busy");
        }
        Err(e) => {
          eprintln!("{}", e);
          return;
        }
      }
      std::thread::sleep(Duration::from_millis(250));
    });
  }

  fn device_left(&mut self, device: rusb::Device<GlobalContext>) {
    if let Some((dev, _)) = DEVICES
      .write()
      .unwrap()
      .remove_entry(&(device.bus_number(), device.address()))
    {
      if let Err(msg) = self.window.emit(
        "removeDevice",
        DevId {
          bus_number: dev.0,
          address: dev.1,
        },
      ) {
        eprintln!("{}", msg);
      };
    }
  }
}

fn err_wrap<T, R: Runtime>(
  res: Result<T, libnspire::Error>,
  dev: DevId,
  window: &Window<R>,
) -> Result<T, libnspire::Error> {
  if let Err(libnspire::Error::NoDevice) = res {
    DEVICES
      .write()
      .unwrap()
      .remove(&(dev.bus_number, dev.address));
    if let Err(msg) = window.emit("removeDevice", dev) {
      eprintln!("{}", msg);
    };
  }
  res
}

fn progress_sender<R: Runtime>(
  window: &Window<R>,
  dev: DevId,
  total: usize,
) -> impl FnMut(usize) + '_ {
  let mut i = 0;
  move |remaining| {
    if i > 5 {
      i = 0;
    }
    if i == 0 || remaining == 0 {
      if let Err(msg) = window.emit(
        "progress",
        ProgressUpdate {
          dev,
          remaining,
          total,
        },
      ) {
        eprintln!("{}", msg);
      };
    }
    i += 1;
  }
}

fn get_open_dev(
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

#[derive(Serialize)]
pub struct SerializedError(String);

impl<T: std::fmt::Display> From<T> for SerializedError {
  fn from(f: T) -> Self {
    SerializedError(f.to_string())
  }
}

mod invoked {
  use std::fs::File;
  use std::io::{Read, Write};
  use std::path::PathBuf;
  use std::sync::{Arc, Mutex};

  use libnspire::dir::EntryType;
  use serde::Serialize;
  use tauri::{Runtime, Window};

  use crate::cmd::{DevId, FileInfo};
  use crate::{err_wrap, get_open_dev, progress_sender, DeviceState, SerializedError};

  use super::DEVICES;

  #[tauri::command]
  pub fn open_device(bus_number: u8, address: u8) -> Result<impl Serialize, SerializedError> {
    let device = if let Some(dev) = DEVICES.read().unwrap().get(&(bus_number, address)) {
      if !matches!(dev.state, DeviceState::Closed) {
        return Err("Already open".into());
      };
      dev.device.clone()
    } else {
      return Err("Failed to find device".into());
    };
    let handle = libnspire::Handle::new(device.open()?)?;
    let info = handle.info()?;
    {
      let mut guard = DEVICES.write().unwrap();
      let device = guard
        .get_mut(&(bus_number, address))
        .ok_or_else(|| anyhow::anyhow!("Device lost"))?;
      device.state = DeviceState::Open(Arc::new(Mutex::new(handle)), info.clone());
    }
    Ok(info)
  }

  #[tauri::command]
  pub fn close_device(bus_number: u8, address: u8) -> Result<impl Serialize, SerializedError> {
    let mut guard = DEVICES.write().unwrap();
    let device = guard
      .get_mut(&(bus_number, address))
      .ok_or_else(|| anyhow::anyhow!("Device lost"))?;
    device.state = DeviceState::Closed;
    Ok(())
  }

  #[tauri::command]
  pub fn update_device<R: Runtime>(
    bus_number: u8,
    address: u8,
    window: Window<R>,
  ) -> Result<impl Serialize, SerializedError> {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    let info = err_wrap(handle.info(), dev, &window)?;
    Ok(info)
  }

  #[tauri::command]
  pub fn list_dir<R: Runtime>(
    bus_number: u8,
    address: u8,
    path: String,
    window: Window<R>,
  ) -> Result<impl Serialize, SerializedError> {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    let dir = err_wrap(handle.list_dir(&path), dev, &window)?;

    Ok(
      dir
        .iter()
        .map(|file| FileInfo {
          path: file.name().to_string_lossy().to_string(),
          is_dir: file.entry_type() == EntryType::Directory,
          date: file.date(),
          size: file.size(),
        })
        .collect::<Vec<_>>(),
    )
  }

  #[tauri::command]
  pub fn download_file<R: Runtime>(
    bus_number: u8,
    address: u8,
    path: (String, u64),
    dest: String,
    window: Window<R>,
  ) -> Result<impl Serialize, SerializedError> {
    let dev = DevId {
      bus_number,
      address,
    };
    let (file, size) = path;
    let dest = PathBuf::from(dest);
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    let mut buf = vec![0; size as usize];
    err_wrap(
      handle.read_file(
        &file,
        &mut buf,
        &mut progress_sender(&window, dev, size as usize),
      ),
      dev,
      &window,
    )?;
    if let Some(name) = file.split('/').last() {
      File::create(dest.join(name))?.write_all(&buf)?;
    }
    Ok(())
  }

  #[tauri::command]
  pub fn upload_file<R: Runtime>(
    bus_number: u8,
    address: u8,
    path: String,
    src: String,
    window: Window<R>,
  ) -> Result<impl Serialize, SerializedError> {
    let dev = DevId {
      bus_number,
      address,
    };
    let file = PathBuf::from(src);
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    let mut buf = vec![];
    File::open(&file)?.read_to_end(&mut buf)?;
    let name = file
      .file_name()
      .ok_or_else(|| anyhow::anyhow!("Failed to get file name"))?
      .to_string_lossy()
      .to_string();
    err_wrap(
      handle.write_file(
        &format!("{}/{}", path, name),
        &buf,
        &mut progress_sender(&window, dev, buf.len()),
      ),
      dev,
      &window,
    )?;
    Ok(())
  }

  #[tauri::command]
  pub fn upload_os<R: Runtime>(
    bus_number: u8,
    address: u8,
    src: String,
    window: Window<R>,
  ) -> Result<impl Serialize, SerializedError> {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    let mut buf = vec![];
    File::open(&src)?.read_to_end(&mut buf)?;
    err_wrap(
      handle.send_os(&buf, &mut progress_sender(&window, dev, buf.len())),
      dev,
      &window,
    )?;
    Ok(())
  }

  #[tauri::command]
  pub fn delete_file<R: Runtime>(
    bus_number: u8,
    address: u8,
    path: String,
    window: Window<R>,
  ) -> Result<impl Serialize, SerializedError> {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    err_wrap(handle.delete_file(&path), dev, &window)?;
    Ok(())
  }

  #[tauri::command]
  pub fn delete_dir<R: Runtime>(
    bus_number: u8,
    address: u8,
    path: String,
    window: Window<R>,
  ) -> Result<impl Serialize, SerializedError> {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    err_wrap(handle.delete_dir(&path), dev, &window)?;
    Ok(())
  }

  #[tauri::command]
  pub fn create_nspire_dir<R: Runtime>(
    bus_number: u8,
    address: u8,
    path: String,
    window: Window<R>,
  ) -> Result<impl Serialize, SerializedError> {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    err_wrap(handle.create_dir(&path), dev, &window)?;
    Ok(())
  }

  #[tauri::command]
  pub fn move_file<R: Runtime>(
    bus_number: u8,
    address: u8,
    src: String,
    dest: String,
    window: Window<R>,
  ) -> Result<impl Serialize, SerializedError> {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    err_wrap(handle.move_file(&src, &dest), dev, &window)?;
    Ok(())
  }

  #[tauri::command]
  pub fn copy<R: Runtime>(
    bus_number: u8,
    address: u8,
    src: String,
    dest: String,
    window: Window<R>,
  ) -> Result<impl Serialize, SerializedError> {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    err_wrap(handle.copy_file(&src, &dest), dev, &window)?;
    Ok(())
  }
}

fn main() {
  if cli::run() {
    return;
  }
  let has_registered_callback = AtomicBool::new(false);
  tauri::Builder::default()
    .on_page_load(move |window, _p| {
      if !has_registered_callback.swap(true, Ordering::SeqCst) {
        if rusb::has_hotplug() {
          if let Err(msg) = GlobalContext::default().register_callback(
            Some(VID),
            None,
            None,
            Box::new(DeviceMon { window }),
          ) {
            eprintln!("{}", msg);
          };
          std::thread::spawn(|| loop {
            GlobalContext::default().handle_events(None).unwrap();
          });
        } else {
          println!("no hotplug");
        }
      }
    })
    .invoke_handler(tauri::generate_handler![
      cmd::enumerate,
      invoked::open_device,
      invoked::close_device,
      invoked::update_device,
      invoked::list_dir,
      invoked::download_file,
      invoked::upload_file,
      invoked::upload_os,
      invoked::delete_file,
      invoked::delete_dir,
      invoked::create_nspire_dir,
      invoked::move_file,
      invoked::copy,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
