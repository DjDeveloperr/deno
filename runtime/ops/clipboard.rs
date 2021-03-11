use deno_clipboard::*;

pub fn init(rt: &mut deno_core::JsRuntime) {
  super::reg_json_sync(
    rt,
    "op_clipboard_new",
    op_clipboard_new,
  );

  super::reg_json_sync(
    rt,
    "op_clipboard_write_text",
    op_clipboard_write_text,
  );

  super::reg_json_sync(
    rt,
    "op_clipboard_read_text",
    op_clipboard_read_text,
  );
}
