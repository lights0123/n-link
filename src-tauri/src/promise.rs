use crate::cmd::Promise;
use serde::Serialize;
use tauri::api::rpc::{format_callback, format_callback_result};
use tauri::Webview;

pub fn promise_fn<R: Serialize, F: FnOnce() -> tauri::Result<R> + Send + 'static>(
  webview: &mut Webview<'_>,
  task: F,
  Promise {
    callback: success_callback,
    error: error_callback,
  }: Promise,
) {
  let mut webview = webview.as_mut();
  std::thread::spawn(move || {
    let callback_string = match format_callback_result(
      task().map_err(|err| err.to_string()),
      success_callback,
      error_callback.clone(),
    ) {
      Ok(callback_string) => callback_string,
      Err(e) => format_callback(error_callback, e.to_string()),
    };
    webview
      .dispatch(move |webview_ref| webview_ref.eval(callback_string.as_str()))
      .expect("Failed to dispatch promise callback");
  });
}
