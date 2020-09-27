// #![cfg_attr(
//   all(not(debug_assertions), target_os = "windows"),
//   windows_subsystem = "windows"
// )]

use hashbrown::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use libnspire::dir::EntryType;
use libnspire::{PID_CX2, VID};
use native_dialog::Dialog;
use rusb::{GlobalContext, Hotplug, UsbContext};
use tauri::WebviewMut;

use crate::cmd::{add_device, AddDevice, DevId, FileInfo, ProgressUpdate};
use crate::promise::promise_fn;

mod cli;
mod cmd;
mod promise;

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
struct DeviceMon {
  handle: WebviewMut,
}

impl Hotplug<GlobalContext> for DeviceMon {
  fn device_arrived(&mut self, device: rusb::Device<GlobalContext>) {
    let mut handle = self.handle.clone();
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
          if let Err(msg) = tauri::event::emit(
            &mut handle,
            "addDevice",
            Some(AddDevice {
              dev: DevId {
                bus_number: (dev.0).0,
                address: (dev.0).1,
              },
              name,
              is_cx_ii,
              needs_drivers,
            }),
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
    // if let Some((dev, _)) = DEVICES
    //   .write()
    //   .unwrap()
    //   .remove_entry(&(device.bus_number(), device.address()))
    // {
    //   if let Err(msg) = tauri::event::emit(
    //     &mut self.handle,
    //     "removeDevice",
    //     Some(DevId {
    //       bus_number: dev.0,
    //       address: dev.1,
    //     }),
    //   ) {
    //     eprintln!("{}", msg);
    //   };
    // }
  }
}

fn err_wrap<T>(
  res: Result<T, libnspire::Error>,
  dev: DevId,
  handle: &mut WebviewMut,
) -> Result<T, libnspire::Error> {
  if let Err(libnspire::Error::NoDevice) = res {
    DEVICES
      .write()
      .unwrap()
      .remove(&(dev.bus_number, dev.address));
    if let Err(msg) = tauri::event::emit(handle, "removeDevice", Some(dev)) {
      eprintln!("{}", msg);
    };
  }
  res
}

fn progress_sender<'a>(
  handle: &'a mut WebviewMut,
  dev: DevId,
  total: usize,
) -> impl FnMut(usize) + 'a {
  let mut i = 0;
  move |remaining| {
    if i > 5 {
      i = 0;
    }
    if i == 0 || remaining == 0 {
      if let Err(msg) = tauri::event::emit(
        handle,
        "progress",
        Some(ProgressUpdate {
          dev,
          remaining,
          total,
        }),
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

fn main() {
  if cli::run() {
    return;
  }
  let mut has_registered_callback = false;
  tauri::AppBuilder::new()
    .invoke_handler(move |webview, arg| {
      use cmd::Cmd::*;
      match serde_json::from_str(arg) {
        Err(e) => Err(e.to_string()),
        Ok(command) => {
          let mut wv_handle = webview.as_mut();
          match command {
            Enumerate { promise } => {
              if !has_registered_callback {
                has_registered_callback = true;
                if rusb::has_hotplug() {
                  if let Err(msg) = GlobalContext::default().register_callback(
                    Some(VID),
                    None,
                    None,
                    Box::new(DeviceMon {
                      handle: webview.as_mut(),
                    }),
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
              promise_fn(
                webview,
                move || Ok(cmd::enumerate(&mut wv_handle)?),
                promise,
              );
            }
            OpenDevice { promise, dev } => {
              promise_fn(
                webview,
                move || {
                  let device = if let Some(dev) =
                    DEVICES.read().unwrap().get(&(dev.bus_number, dev.address))
                  {
                    anyhow::ensure!(matches!(dev.state, DeviceState::Closed), "Already open");
                    dev.device.clone()
                  } else {
                    anyhow::bail!("Failed to find device");
                  };
                  let handle = libnspire::Handle::new(device.open()?)?;
                  let info = handle.info()?;
                  {
                    let mut guard = DEVICES.write().unwrap();
                    let device = guard
                      .get_mut(&(dev.bus_number, dev.address))
                      .ok_or_else(|| anyhow::anyhow!("Device lost"))?;
                    device.state = DeviceState::Open(Arc::new(Mutex::new(handle)), info.clone());
                  }
                  Ok(info)
                },
                promise,
              );
            }
            CloseDevice { promise, dev } => {
              promise_fn(
                webview,
                move || {
                  {
                    let mut guard = DEVICES.write().unwrap();
                    let device = guard
                      .get_mut(&(dev.bus_number, dev.address))
                      .ok_or_else(|| anyhow::anyhow!("Device lost"))?;
                    device.state = DeviceState::Closed;
                  }
                  Ok(())
                },
                promise,
              );
            }
            UpdateDevice { promise, dev } => {
              promise_fn(
                webview,
                move || {
                  let handle = get_open_dev(&dev)?;
                  let handle = handle.lock().unwrap();
                  let info = err_wrap(handle.info(), dev, &mut wv_handle)?;
                  Ok(info)
                },
                promise,
              );
            }
            ListDir { promise, dev, path } => {
              promise_fn(
                webview,
                move || {
                  let handle = get_open_dev(&dev)?;
                  let handle = handle.lock().unwrap();
                  let dir = err_wrap(handle.list_dir(&path), dev, &mut wv_handle)?;

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
                },
                promise,
              );
            }
            DownloadFile {
              promise,
              dev,
              path: (file, size),
              dest,
            } => {
              promise_fn(
                webview,
                move || {
                  let dest = PathBuf::from(dest);
                  let handle = get_open_dev(&dev)?;
                  let handle = handle.lock().unwrap();
                  let mut buf = vec![0; size as usize];
                  err_wrap(
                    handle.read_file(
                      &file,
                      &mut buf,
                      &mut progress_sender(&mut wv_handle.clone(), dev, size as usize),
                    ),
                    dev,
                    &mut wv_handle,
                  )?;
                  if let Some(name) = file.split('/').last() {
                    File::create(dest.join(name))?.write_all(&buf)?;
                  }
                  Ok(())
                },
                promise,
              );
            }
            UploadFile {
              promise,
              dev,
              path,
              src,
            } => {
              promise_fn(
                webview,
                move || {
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
                      &mut progress_sender(&mut wv_handle.clone(), dev, buf.len()),
                    ),
                    dev,
                    &mut wv_handle,
                  )?;
                  Ok(())
                },
                promise,
              );
            }
            UploadOs { promise, dev, src } => {
              promise_fn(
                webview,
                move || {
                  let handle = get_open_dev(&dev)?;
                  let handle = handle.lock().unwrap();
                  let mut buf = vec![];
                  File::open(&src)?.read_to_end(&mut buf)?;
                  err_wrap(
                    handle.send_os(
                      &buf,
                      &mut progress_sender(&mut wv_handle.clone(), dev, buf.len()),
                    ),
                    dev,
                    &mut wv_handle,
                  )?;
                  Ok(())
                },
                promise,
              );
            }
            DeleteFile { promise, dev, path } => {
              promise_fn(
                webview,
                move || {
                  let handle = get_open_dev(&dev)?;
                  let handle = handle.lock().unwrap();
                  err_wrap(handle.delete_file(&path), dev, &mut wv_handle)?;
                  Ok(())
                },
                promise,
              );
            }
            DeleteDir { promise, dev, path } => {
              promise_fn(
                webview,
                move || {
                  let handle = get_open_dev(&dev)?;
                  let handle = handle.lock().unwrap();
                  err_wrap(handle.delete_dir(&path), dev, &mut wv_handle)?;
                  Ok(())
                },
                promise,
              );
            }
            CreateNspireDir { promise, dev, path } => {
              promise_fn(
                webview,
                move || {
                  let handle = get_open_dev(&dev)?;
                  let handle = handle.lock().unwrap();
                  err_wrap(handle.create_dir(&path), dev, &mut wv_handle)?;
                  Ok(())
                },
                promise,
              );
            }
            Move {
              promise,
              dev,
              src,
              dest,
            } => {
              promise_fn(
                webview,
                move || {
                  let handle = get_open_dev(&dev)?;
                  let handle = handle.lock().unwrap();
                  err_wrap(handle.move_file(&src, &dest), dev, &mut wv_handle)?;
                  Ok(())
                },
                promise,
              );
            }
            Copy {
              promise,
              dev,
              src,
              dest,
            } => {
              promise_fn(
                webview,
                move || {
                  let handle = get_open_dev(&dev)?;
                  let handle = handle.lock().unwrap();
                  err_wrap(handle.copy_file(&src, &dest), dev, &mut wv_handle)?;
                  Ok(())
                },
                promise,
              );
            }
            SelectFile { promise, filter } => {
              promise_fn(
                webview,
                move || {
                  let filter = filter.iter().map(|t| t.as_str()).collect::<Vec<_>>();
                  Ok(
                    (native_dialog::OpenSingleFile {
                      filter: Some(&filter),
                      dir: None,
                    })
                    .show()?,
                  )
                },
                promise,
              );
            }
            SelectFiles { promise, filter } => {
              promise_fn(
                webview,
                move || {
                  let filter = filter.iter().map(|t| t.as_str()).collect::<Vec<_>>();
                  Ok(
                    (native_dialog::OpenMultipleFile {
                      filter: Some(&filter),
                      dir: None,
                    })
                    .show()?,
                  )
                },
                promise,
              );
            }
            SelectFolder { promise } => {
              promise_fn(
                webview,
                move || Ok((native_dialog::OpenSingleDir { dir: None }).show()?),
                promise,
              );
            }
          }
          Ok(())
        }
      }
    })
    .build()
    .run();
}
