use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::Clap;
use indicatif::{ProgressBar, ProgressStyle};
use libnspire::{PID, PID_CX2, VID};

#[derive(Clap, Debug)]
#[clap(author, about, version)]
struct Opt {
  #[clap(subcommand)]
  cmd: Option<SubCommand>,
}

#[derive(Clap, Debug)]
enum SubCommand {
  Upload(Upload),
  /// View license information
  License,
  // Download(Download),
}

/// Upload files to the calculator
#[derive(Clap, Debug)]
struct Upload {
  /// Files to upload
  #[clap(required = true, parse(from_os_str))]
  files: Vec<PathBuf>,
  /// Destination path
  dest: String,
}

// /// Download files from the calculator
// #[derive(Clap, Debug)]
// struct Download {
//   /// Files to download
//   #[clap(required = true)]
//   files: Vec<String>,
//   /// Destination path
//   #[clap(parse(from_os_str))]
//   dest: PathBuf,
// }

fn get_dev() -> Option<libnspire::Handle<rusb::GlobalContext>> {
  rusb::devices()
    .unwrap()
    .iter()
    .filter(|dev| {
      let descriptor = match dev.device_descriptor() {
        Ok(d) => d,
        Err(_) => return false,
      };
      descriptor.vendor_id() == VID && matches!(descriptor.product_id(), PID | PID_CX2)
    })
    .next()
    .map(|dev| libnspire::Handle::new(dev.open().unwrap()).unwrap())
}

pub fn cwd() -> PathBuf {
  #[cfg(target_os = "linux")]
  if std::env::var_os("APPIMAGE").is_some() && std::env::var_os("APPDIR").is_some() {
    if let Some(cwd) = std::env::var_os("OWD") {
      return cwd.into();
    }
  };
  std::env::current_dir().expect("Couldn't get current directory")
}

pub fn run() -> bool {
  let opt: Opt = Opt::parse();
  if let Some(cmd) = opt.cmd {
    match cmd {
      SubCommand::Upload(Upload { files, mut dest }) => {
        if let Some(handle) = get_dev() {
          for file in files {
            let mut buf = vec![];
            File::open(cwd().join(&file))
              .unwrap()
              .read_to_end(&mut buf)
              .unwrap();
            let name = file
              .file_name()
              .expect("Failed to get file name")
              .to_string_lossy()
              .to_string();
            let bar = ProgressBar::new(buf.len() as u64);
            bar.set_style(ProgressStyle::default_bar().template("{spinner:.green} {msg}[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"));
            bar.set_message(&format!("Upload {}", name));
            bar.enable_steady_tick(100);
            if dest.ends_with('/') {
              dest.remove(dest.len() - 1);
            }
            handle
              .write_file(&format!("{}/{}", dest, name), &buf, &mut |remaining| {
                bar.set_position((buf.len() - remaining) as u64)
              })
              .unwrap();
            bar.finish();
          }
        } else {
          eprintln!("Couldn't find any device");
        }
      }
      SubCommand::License => {
        println!("{}", include_str!("../../LICENSE"));
        println!(include_str!("NOTICE.txt"), env!("CARGO_PKG_REPOSITORY"));
      }
    }
    true
  } else {
    false
  }
}
