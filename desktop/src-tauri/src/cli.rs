use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{fs::File, path::Path};

use clap::Clap;
use indicatif::{ProgressBar, ProgressStyle};
use libnspire::{dir::EntryType, PID, PID_CX2, VID};

#[derive(Clap, Debug)]
#[clap(author, about, version)]
struct Opt {
  #[clap(subcommand)]
  cmd: Option<SubCommand>,
}

#[derive(Clap, Debug)]
enum SubCommand {
  Upload(Upload),
  Download(Download),
  UploadOS(UploadOS),
  Copy(Copy),
  Move(Move),
  Mkdir(Mkdir),
  Rmdir(Rmdir),
  Ls(Ls),
  /// View license information
  License,
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

/// Download files from the calculator
#[derive(Clap, Debug)]
struct Download {
  /// Files to download
  #[clap(required = true)]
  files: Vec<String>,
  /// Destination path
  #[clap(required = true, parse(from_os_str))]
  dest: PathBuf,
}

/// Upload and install a .tcc/.tco/.tcc2/.tco2/.tct2 OS file
#[derive(Clap, Debug)]
struct UploadOS {
  /// Path to the OS file
  #[clap(required = true, parse(from_os_str))]
  file: PathBuf,

  /// Disables the file extension check
  #[clap(long)]
  no_check_os: bool,
}

/// Copy a file to a different location
#[derive(Clap, Debug)]
struct Copy {
  /// Path to file
  #[clap(required = true)]
  from_path: String,

  /// Path to new location
  #[clap(required = true)]
  dist_path: String,
}

/// Move a file or directory to a new location
#[derive(Clap, Debug)]
struct Move {
  /// Path to file
  #[clap(required = true)]
  from_path: String,

  /// Path to new location
  #[clap(required = true)]
  dist_path: String,
}

/// Create a directory
#[derive(Clap, Debug)]
struct Mkdir {
  /// Path to directory
  #[clap(required = true)]
  path: String,
}

/// Delete a directory
#[derive(Clap, Debug)]
struct Rmdir {
  /// Path to directory
  #[clap(required = true)]
  path: String,
}

/// List the contents of a directory
#[derive(Clap, Debug)]
struct Ls {
  /// Path to directory
  #[clap(required = true)]
  path: String,
}

fn get_dev() -> Option<libnspire::Handle<rusb::GlobalContext>> {
  rusb::devices()
    .unwrap()
    .iter()
    .find(|dev| {
      let descriptor = match dev.device_descriptor() {
        Ok(d) => d,
        Err(_) => return false,
      };
      descriptor.vendor_id() == VID && matches!(descriptor.product_id(), PID | PID_CX2)
    })
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
            bar.set_style(ProgressStyle::default_bar().template("{spinner:.green} {msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"));
            bar.set_message(&format!("Upload {}", name));
            bar.enable_steady_tick(100);
            if dest.ends_with('/') {
              dest.remove(dest.len() - 1);
            }
            let res = handle.write_file(&format!("{}/{}", dest, name), &buf, &mut |remaining| {
              bar.set_position((buf.len() - remaining) as u64)
            });

            match res {
              Ok(_) => {
                bar.finish_with_message(&format!("Upload {}: Ok", dest));
              }
              Err(error) => {
                bar.abandon_with_message(&format!("Failed: {}", error));
              }
            }
          }
        } else {
          eprintln!("Couldn't find any device");
        }
      }
      SubCommand::Download(Download { dest, files }) => {
        if let Some(handle) = get_dev() {
          for file in files {
            let attr = handle.file_attr(&file);
            match attr {
              Ok(attr) => {
                let path = Path::new(&file);
                let dest_path = Path::join(&dest, path.file_name().unwrap().to_str().unwrap());
                match File::create(dest_path) {
                  Ok(mut dest_file) => {
                    let mut buf = vec![0u8; attr.size() as usize];

                    let bar = ProgressBar::new(buf.len() as u64);
                    bar.set_style(ProgressStyle::default_bar().template("{spinner:.green} {msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"));
                    bar.set_message(&format!(
                      "Download {}",
                      path.file_name().unwrap().to_str().unwrap()
                    ));
                    bar.enable_steady_tick(100);

                    let len = buf.len();

                    let res = handle.read_file(&file, &mut buf, &mut |remaining| {
                      bar.set_position((len - remaining) as u64);
                    });

                    match res {
                      Ok(_) => {
                        bar.set_message("Writing file to disk");

                        match dest_file.write_all(&buf) {
                          Ok(_) => {
                            bar.finish_with_message("Transfer completed");
                          }
                          Err(error) => {
                            bar.abandon_with_message(&format!(
                              "Failed to write file to disk: {}",
                              error
                            ));
                          }
                        }
                      }
                      Err(error) => {
                        bar.abandon_with_message(&format!("Failed to transfer file: {}", error))
                      }
                    }
                  }
                  Err(error) => {
                    eprintln!("Failed to open destination file: {}", error);
                  }
                }
              }
              Err(error) => {
                eprintln!("Failed to read file info: {}", error);
              }
            }
          }
        } else {
          eprintln!("Couldn't find any device");
        }
      }
      SubCommand::UploadOS(UploadOS { file, no_check_os }) => {
        if let Some(handle) = get_dev() {
          let calc_info = handle.info().expect("Failed to obtain device info");

          let file_ext = file
            .extension()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
            .to_string();

          let mut buf = vec![];
          let mut f = File::open(cwd().join(&file)).unwrap_or_else(|err| {
            eprintln!("Failed to open file: {}", err);
            std::process::exit(1);
          });

          if format!(".{}", file_ext) != calc_info.os_extension {
            if no_check_os {
              eprintln!(
                "Warning: {} expects file of type {}",
                calc_info.name, calc_info.os_extension
              );
            } else {
              eprintln!(
                "Error: {} expects file of type {}",
                calc_info.name, calc_info.os_extension
              );
              eprintln!("Provide --no-check-os to bypass this check.");
              std::process::exit(1);
            }
          }

          f.read_to_end(&mut buf).unwrap();

          let name = file
            .file_name()
            .expect("Failed to get file name")
            .to_string_lossy()
            .to_string();

          let bar = ProgressBar::new(buf.len() as u64);
          bar.set_style(ProgressStyle::default_bar().template("{spinner:.green} {msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"));
          bar.set_message(&format!("Upload OS {}", name));
          bar.enable_steady_tick(100);

          let res = handle.send_os(&buf, &mut |remaining| {
            bar.set_position((buf.len() - remaining) as u64);
          });

          match res {
            Ok(_) => {
              bar.finish();
            }
            Err(error) => {
              bar.abandon_with_message(&format!("OS Upload failed: {}", error));
            }
          }
        } else {
          eprintln!("Couldn't find any device");
        }
      }
      SubCommand::Copy(Copy {
        from_path,
        dist_path,
      }) => {
        if let Some(handle) = get_dev() {
          match handle.copy_file(&from_path, &dist_path) {
            Ok(_) => {
              println!("Copy {} => {}: Ok", from_path, dist_path);
            }
            Err(error) => {
              eprintln!("Failed to copy file or directory: {}", error);
            }
          }
        } else {
          eprintln!("Couldn't find any device");
        }
      }
      SubCommand::Move(Move {
        from_path,
        dist_path,
      }) => {
        if let Some(handle) = get_dev() {
          match handle.move_file(&from_path, &dist_path) {
            Ok(_) => {
              println!("Move {} => {}: Ok", from_path, dist_path);
            }
            Err(error) => {
              eprintln!("Failed to move file or directory: {}", error);
            }
          }
        } else {
          eprintln!("Couldn't find any device");
        }
      }
      SubCommand::Mkdir(Mkdir { path }) => {
        if let Some(handle) = get_dev() {
          match handle.create_dir(&path) {
            Ok(_) => {
              println!("Create {}: Ok", path);
            }
            Err(error) => {
              eprintln!("Failed to create directory: {}", error);
            }
          }
        } else {
          eprintln!("Couldn't find any device");
        }
      }
      SubCommand::Rmdir(Rmdir { path }) => {
        if let Some(handle) = get_dev() {
          match handle.delete_dir(&path) {
            Ok(_) => {
              println!("Remove {}: Ok", path);
            }
            Err(error) => {
              eprintln!("Failed to delete directory: {}", error);
            }
          }
        } else {
          eprintln!("Couldn't find any device");
        }
      }
      SubCommand::Ls(Ls { path }) => {
        if let Some(handle) = get_dev() {
          match handle.list_dir(&path) {
            Ok(dir_list) => {
              for item in dir_list.iter() {
                println!(
                  "{}{}",
                  item.name().to_str().unwrap(),
                  if item.entry_type() == EntryType::Directory {
                    "/"
                  } else {
                    ""
                  }
                );
              }
            }
            Err(error) => {
              eprintln!("Failed to list directory: {}", error);
            }
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
