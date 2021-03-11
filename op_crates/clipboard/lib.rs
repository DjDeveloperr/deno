use deno_core::serde_json::Value;
use deno_core::serde_json::json;
use deno_core::JsRuntime;
use deno_core::error::AnyError;
use deno_core::OpState;
use deno_core::{ZeroCopyBuf};
use deno_core::ResourceId;
use arboard::*;
use deno_core::Resource;
use deno_core::error::bad_resource_id;

use std::path::PathBuf;
use std::borrow::Cow;
use std::cell::RefCell;

struct WebClipboard(RefCell<Clipboard>);

impl Resource for WebClipboard {
  fn name(&self) -> Cow<str> {
    "Clipboard".into()
  }
}

pub fn op_clipboard_new(
  state: &mut OpState,
  _args: Value,
  _zero_copy: &mut [ZeroCopyBuf],
) -> Result<Value, AnyError> {
  let rid = state.resource_table.add(WebClipboard(RefCell::from(Clipboard::new().unwrap())));
  Ok(json!(rid))
}

pub fn op_clipboard_read_text(
  state: &mut OpState,
  args: Value,
  _zero_copy: &mut [ZeroCopyBuf],
) -> Result<Value, AnyError> {
  let rid = args.get("rid");
  if rid.is_none() {
    return Ok(json!(""));
  }
  let rid = rid.unwrap().as_u64().unwrap() as ResourceId;
  let clipboard_resource = state.resource_table.get::<WebClipboard>(rid).ok_or_else(bad_resource_id)?;
  let mut clipboard = clipboard_resource.0.borrow_mut();

  let read_result = clipboard.get_text();

  if read_result.is_err() {
    Ok(json!(""))
  } else {
    Ok(json!(read_result.unwrap()))
  }
}

pub fn op_clipboard_write_text(
  state: &mut OpState,
  args: Value,
  _zero_copy: &mut [ZeroCopyBuf],
) -> Result<Value, AnyError> {
  let rid = args.get("rid");
  if rid.is_none() {
    return Ok(json!("no rid"));
  }
  let rid = rid.unwrap().as_u64().unwrap() as ResourceId;
  let clipboard_resource = state.resource_table.get::<WebClipboard>(rid).ok_or_else(bad_resource_id)?;
  let mut clipboard = clipboard_resource.0.borrow_mut();

  let get_res = args.get("text");
  if get_res.is_none() { return Ok(json!("no text given")); }
  let arg_res = get_res.unwrap().as_str();
  if arg_res.is_none() {
    Ok(json!("no text given"))
  } else {
    let write_result = clipboard.set_text(String::from(arg_res.unwrap()));

    if write_result.is_err() {
      Ok(json!("success"))
    } else {
      Ok(json!("failed to write"))
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
