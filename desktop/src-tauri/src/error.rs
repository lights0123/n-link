use serde::Serialize;

/// Wraps any displayable error into a serde-serializable form so it can
/// cross the Tauri IPC boundary.
#[derive(Serialize)]
pub struct SerializedError(String);

impl<T: std::fmt::Display> From<T> for SerializedError {
  fn from(f: T) -> Self {
    SerializedError(f.to_string())
  }
}
