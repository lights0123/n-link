use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;

use libnspire::dir::EntryType;
use libnspire::PID_CX2;
use serde::Serialize;
use tauri::{Emitter, Runtime, WebviewWindow};

use crate::device::{
  add_device, get_open_dev, AddDevice, DevId, DeviceState, FileInfo, ProgressUpdate, DEVICES,
};
use crate::error::SerializedError;

/// If the device vanished mid-operation, drop it from the registry and tell
/// the frontend, then pass the original result through unchanged.
fn err_wrap<T, R: Runtime>(
  res: Result<T, libnspire::Error>,
  dev: DevId,
  window: &WebviewWindow<R>,
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

/// Emits a `progress` event on every 6th callback, and always on completion,
/// to avoid flooding the IPC channel during large transfers.
fn progress_sender<R: Runtime>(
  window: &WebviewWindow<R>,
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

#[tauri::command]
pub fn enumerate<R: Runtime>(window: WebviewWindow<R>) -> Result<Vec<AddDevice>, SerializedError> {
  let devices: Vec<_> = rusb::devices()?.iter().collect();
  let mut map = DEVICES.write().unwrap();

  // Replaces hashbrown's `drain_filter`: collect the keys of devices that are
  // no longer present on the bus, then remove them. std HashMap has no stable
  // drain-with-predicate, and this is clearer than the original anyway.
  let stale: Vec<(u8, u8)> = map
    .keys()
    .filter(|k| {
      devices
        .iter()
        .all(|d| d.bus_number() != k.0 || d.address() != k.1)
    })
    .copied()
    .collect();

  for key in stale {
    map.remove(&key);
    if let Err(msg) = window.emit(
      "removeDevice",
      DevId {
        bus_number: key.0,
        address: key.1,
      },
    ) {
      eprintln!("{}", msg);
    }
  }

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

#[tauri::command]
pub async fn open_device(
  bus_number: u8,
  address: u8,
) -> Result<libnspire::info::Info, SerializedError> {
  tauri::async_runtime::spawn_blocking(move || {
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
      device.state = DeviceState::Open(Arc::new(std::sync::Mutex::new(handle)), info.clone());
    }
    Ok(info)
  })
  .await
  .map_err(|e| SerializedError::from(e))?
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
pub async fn update_device<R: Runtime>(
  bus_number: u8,
  address: u8,
  window: WebviewWindow<R>,
) -> Result<libnspire::info::Info, SerializedError> {
  tauri::async_runtime::spawn_blocking(move || {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    let info = err_wrap(handle.info(), dev, &window)?;
    Ok(info)
  })
  .await
  .map_err(|e| SerializedError::from(e))?
}

#[tauri::command]
pub async fn list_dir<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: String,
  window: WebviewWindow<R>,
) -> Result<Vec<FileInfo>, SerializedError> {
  tauri::async_runtime::spawn_blocking(move || {
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
  })
  .await
  .map_err(|e| SerializedError::from(e))?
}

#[tauri::command]
pub async fn download_file<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: (String, u64),
  dest: String,
  window: WebviewWindow<R>,
) -> Result<(), SerializedError> {
  tauri::async_runtime::spawn_blocking(move || {
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
  })
  .await
  .map_err(|e| SerializedError::from(e))?
}

#[tauri::command]
pub async fn upload_file<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: String,
  src: String,
  window: WebviewWindow<R>,
) -> Result<(), SerializedError> {
  tauri::async_runtime::spawn_blocking(move || {
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
  })
  .await
  .map_err(|e| SerializedError::from(e))?
}

#[tauri::command]
pub async fn upload_os<R: Runtime>(
  bus_number: u8,
  address: u8,
  src: String,
  window: WebviewWindow<R>,
) -> Result<(), SerializedError> {
  tauri::async_runtime::spawn_blocking(move || {
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
  })
  .await
  .map_err(|e| SerializedError::from(e))?
}

#[tauri::command]
pub async fn delete_file<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: String,
  window: WebviewWindow<R>,
) -> Result<(), SerializedError> {
  tauri::async_runtime::spawn_blocking(move || {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    err_wrap(handle.delete_file(&path), dev, &window)?;
    Ok(())
  })
  .await
  .map_err(|e| SerializedError::from(e))?
}

#[tauri::command]
pub async fn delete_dir<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: String,
  window: WebviewWindow<R>,
) -> Result<(), SerializedError> {
  tauri::async_runtime::spawn_blocking(move || {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    err_wrap(handle.delete_dir(&path), dev, &window)?;
    Ok(())
  })
  .await
  .map_err(|e| SerializedError::from(e))?
}

#[tauri::command]
pub async fn create_nspire_dir<R: Runtime>(
  bus_number: u8,
  address: u8,
  path: String,
  window: WebviewWindow<R>,
) -> Result<(), SerializedError> {
  tauri::async_runtime::spawn_blocking(move || {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    err_wrap(handle.create_dir(&path), dev, &window)?;
    Ok(())
  })
  .await
  .map_err(|e| SerializedError::from(e))?
}

#[tauri::command]
pub async fn move_file<R: Runtime>(
  bus_number: u8,
  address: u8,
  src: String,
  dest: String,
  window: WebviewWindow<R>,
) -> Result<(), SerializedError> {
  tauri::async_runtime::spawn_blocking(move || {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    err_wrap(handle.move_file(&src, &dest), dev, &window)?;
    Ok(())
  })
  .await
  .map_err(|e| SerializedError::from(e))?
}

#[tauri::command]
pub async fn copy<R: Runtime>(
  bus_number: u8,
  address: u8,
  src: String,
  dest: String,
  window: WebviewWindow<R>,
) -> Result<(), SerializedError> {
  tauri::async_runtime::spawn_blocking(move || {
    let dev = DevId {
      bus_number,
      address,
    };
    let handle = get_open_dev(&dev)?;
    let handle = handle.lock().unwrap();
    err_wrap(handle.copy_file(&src, &dest), dev, &window)?;
    Ok(())
  })
  .await
  .map_err(|e| SerializedError::from(e))?
}
