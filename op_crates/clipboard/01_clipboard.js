// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.
"use strict";

((window) => {
  const core = window.Deno.core;

  class ClipboardItem {
    // what do i write here
  }

  class Clipboard {
    #rid;

    constructor() {
      if (!this.#rid) this.#rid = core.jsonOpSync("op_clipboard_new");
    }

    read() {}
    write() {}

    readText() {
      return new Promise((resolve, reject) => {
        // TODO: Check permission
        let textData = "";
        const result = core.jsonOpSync("op_clipboard_read_text", {
          rid: this.#rid
        });
        if (result) textData = result;
        resolve(textData);
      });
    }

    writeText(text) {
      return new Promise((resolve, reject) => {
        // TODO: Check permission
        core.jsonOpSync("op_clipboard_write_text", {
          rid: this.#rid,
          text,
        });
        resolve();
      });
    }
  }

  window.__bootstrap.clipboard = { clipboard: new Clipboard(), Clipboard, ClipboardItem };
})(this);
