#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use std::time::Duration;

use libnspire::{PID_CX2, VID};
use rusb::{GlobalContext, Hotplug, UsbContext};
use tauri::{Emitter, Manager, Runtime, WebviewWindow};

use crate::device::{add_device, AddDevice, DevId, DEVICES};

mod commands;
mod device;
mod error;

struct DeviceMon<R: Runtime> {
  window: WebviewWindow<R>,
}

impl<R: Runtime> Hotplug<GlobalContext> for DeviceMon<R> {
  fn device_arrived(&mut self, device: rusb::Device<GlobalContext>) {
    let handle = self.window.clone();
    let is_cx_ii = device
      .device_descriptor()
      .map(|d| d.product_id() == PID_CX2)
      .unwrap_or(false);
    let device = Arc::new(device);
    // The calculator often reports Busy immediately after enumeration, so
    // retry until it settles.
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

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_shell::init())
    .setup(|app| {
      // Tauri 1 registered this in on_page_load, guarded by an AtomicBool to
      // avoid double-registering on reload. In v2, setup runs exactly once,
      // so the guard is unnecessary.
      let window = app
        .get_webview_window("main")
        .expect("main window not found");
      if rusb::has_hotplug() {
        if let Err(msg) =
          GlobalContext::default().register_callback(Some(VID), None, None, Box::new(DeviceMon { window }))
        {
          eprintln!("{}", msg);
        };
        std::thread::spawn(|| loop {
          GlobalContext::default().handle_events(None).unwrap();
        });
      } else {
        println!("no hotplug");
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::enumerate,
      commands::open_device,
      commands::close_device,
      commands::update_device,
      commands::list_dir,
      commands::download_file,
      commands::upload_file,
      commands::upload_os,
      commands::delete_file,
      commands::delete_dir,
      commands::create_nspire_dir,
      commands::move_file,
      commands::copy,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
