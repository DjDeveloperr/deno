use deno_core::serde_json::Value;
use deno_core::serde_json::json;
use deno_core::JsRuntime;
use deno_core::error::AnyError;
use deno_core::OpState;
use deno_core::{ZeroCopyBuf};
use arboard::*;

use std::path::PathBuf;

pub fn op_clipboard_read_text(
  _state: &mut OpState,
  _args: Value,
  _zero_copy: &mut [ZeroCopyBuf],
) -> Result<Value, AnyError> {
  let clipboard = Clipboard::new();
  if clipboard.is_err() {
    Ok(json!(""))
  } else {
    let read_result = clipboard.unwrap().get_text();

    if read_result.is_err() {
      Ok(json!(""))
    } else {
      Ok(json!(read_result.unwrap()))
    }
  }
}

pub fn op_clipboard_write_text(
  _state: &mut OpState,
  args: Value,
  _zero_copy: &mut [ZeroCopyBuf],
) -> Result<Value, AnyError> {
  let clipboard = Clipboard::new();
  
  if clipboard.is_err() {
    Ok(json!("1"))
  } else {
    let arg_res = args.as_str();
    if arg_res.is_none() {
      Ok(json!("3"))
    } else {
      let write_result = clipboard.unwrap().set_text(String::from(arg_res.unwrap()));

      if write_result.is_err() {
        Ok(json!("2"))
      } else {
        Ok(json!("0"))
      }
    }
  }
}

/// Load and execute the javascript code.
pub fn init(isolate: &mut JsRuntime) {
  isolate
    .execute(
      "deno:op_crates/clipboard/01_clipboard.js",
      include_str!("01_clipboard.js"),
    )
    .unwrap();

  isolate
    .execute(
      "deno:op_crates/clipboard/02_idl_types.js",
      include_str!("02_idl_types.js"),
    )
    .unwrap();
}

pub fn get_declaration() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib.deno_clipboard.d.ts")
}
