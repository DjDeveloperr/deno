// Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.

/// <reference path="../../core/internal.d.ts" />

import { EventTarget } from "ext:deno_web/02_event.js";

const core = globalThis.Deno.core;
const ops = core.ops;
import * as webidl from "ext:deno_webidl/00_webidl.js";
import DOMException from "ext:deno_web/01_dom_exception.js";
const primordials = globalThis.__bootstrap.primordials;

class Serial extends EventTarget {
  constructor() {
    webidl.illegalConstructor();
  }

  getPorts() {
    webidl.assertBranded(this, SerialPrototype);

    const ports = ops
      .op_webserial_get_ports()
      .map((info) => {
        const port = webidl.createBranded(SerialPort);

        port[_innerInfo] = info;
        port[_state] = "closed";
        port[_readable] = null;
        port[_readFatal] = false;
        port[_writable] = null;
        port[_writeFatal] = false;
        port[_pendingClosePromise] = null;

        return port;
      });
    return Promise.resolve(ports);
  }
}

const SerialPrototype = Serial.prototype;

const _innerInfo = Symbol("[[innerInfo]]");
const _rid = Symbol("[[rid]]");
const _state = Symbol("[[state]]");
const _bufferSize = Symbol("[[bufferSize]]");
const _readable = Symbol("[[readable]]");
const _readFatal = Symbol("[[readFatal]]");
const _writable = Symbol("[[writable]]");
const _writeFatal = Symbol("[[writeFatal]]");
const _pendingClosePromise = Symbol("[[pendingClosePromise]]");

class Deferred {
  promise;
  resolve;
  reject;

  constructor() {
    this.promise = new Promise((resolve, reject) => {
      this.resolve = resolve;
      this.reject = reject;
    });
  }
}

class SerialPort extends EventTarget {
  [_innerInfo];
  [_rid];

  [_state];
  [_bufferSize];
  [_readable];
  [_readFatal];
  [_writable];
  [_writeFatal];
  [_pendingClosePromise];

  constructor() {
    webidl.illegalConstructor();
  }

  getInfo() {
    webidl.assertBranded(this, SerialPortPrototype);

    if (this[_innerInfo].usbVendorId === null) {
      return {
        name: this[_innerInfo].name,
      };
    } else {
      return {
        name: this[_innerInfo].name,
        usbVendorId: this[_innerInfo].usbVendorId,
        usbProductId: this[_innerInfo].usbProductId,
      }
    }
  }

  open(options) {
    webidl.assertBranded(this, SerialPortPrototype);

    if (this[_state] !== "closed") {
      throw new DOMException(
        "Port is not closed",
        "InvalidStateError",
      );
    }

    if (options.dataBits !== undefined && options.dataBits !== 8 && options.dataBits !== 7) {
      throw new TypeError("Invalid dataBits, must be 7 or 8");
    }

    if (options.stopBits !== undefined && options.stopBits !== 1 && options.stopBits !== 2) {
      throw new TypeError("Invalid stopBits, must be 1 or 2");
    }

    if (options.bufferSize === 0) {
      throw new TypeError("Invalid bufferSize, must be greater than 0");
    }

    this[_state] = "opening";

    this[_rid] = ops.op_webserial_open(
      this[_innerInfo].name,
      options.baudRate,
      options.dataBits ?? 8,
      options.stopBits ?? 1,
      options.parity ?? "none",
      options.flowControl ?? "none",
    );

    this[_state] = "opened";
    this[_bufferSize] = options.bufferSize ?? 255;

    return Promise.resolve();
  }

  get readable() {
    webidl.assertBranded(this, SerialPortPrototype);

    if (this[_readable] !== null) {
      return this[_readable];
    }

    if (this[_state] !== "opened") {
      throw new DOMException(
        "Port is not opened",
        "InvalidStateError",
      );
    }

    if (this[_readFatal]) {
      return null;
    }

    const highWaterMark = this[_bufferSize];

    this[_readable] = new ReadableStream({
      type: "bytes",

      pull: async (controller) => {
        if (controller.byobRequest) {
          if (controller.byobRequest.view) {
            const buffer = new Uint8Array(
              controller.byobRequest.view.buffer,
              controller.byobRequest.view.byteOffset,
              controller.byobRequest.view.byteLength,
            );
            const nread = await ops.op_webserial_read(this[_rid], buffer);
            if (nread === 0) {
              controller.close();
              return;
            }
            controller.byobRequest.respond(nread);
          } else {
            const buffer = new Uint8Array(
              controller.desiredSize ?? highWaterMark,
            );
            const nread = await ops.op_webserial_read(this[_rid], buffer);
            if (nread === 0) {
              controller.close();
              return;
            }
            controller.byobRequest.respondWithNewView(
              buffer.subarray(0, nread),
            );
          }
        } else {
          const buffer = new Uint8Array(
            controller.desiredSize ?? highWaterMark,
          );
          const nread = await ops.op_webserial_read(this[_rid], buffer);
          if (nread === 0) {
            controller.close();
            return;
          }
          controller.enqueue(buffer.subarray(0, nread));
        }
      },

      cancel: (c) => {
        this[_readable] = undefined;
        if (this[_writable] === null && this[_pendingClosePromise]) {
          this[_pendingClosePromise].resolve();
        }
        return Promise.resolve();
      },
    }, { highWaterMark });

    return this[_readable];
  }

  get writable() {
    webidl.assertBranded(this, SerialPortPrototype);

    if (this[_writable] !== null) {
      return this[_writable];
    }

    if (this[_state] !== "opened") {
      throw new DOMException(
        "Port is not opened",
        "InvalidStateError",
      );
    }

    if (this[_writeFatal]) {
      return null;
    }

    const highWaterMark = this[_bufferSize];

    this[_writable] = new WritableStream({
      write: async (chunk) => {
        await ops.op_webserial_write(this[_rid], chunk);
      },
      
      close: () => {
        console.log("wclose")
        this[_writable] = undefined;
        if (this[_readable] === null && this[_pendingClosePromise]) {
          this[_pendingClosePromise].resolve();
        }
        return Promise.resolve();
      },
      
      abort: (reason) => {
        console.log("wabort")
        this[_writable] = undefined;
        if (this[_readable] === null && this[_pendingClosePromise]) {
          this[_pendingClosePromise].resolve();
        }
        return Promise.resolve();
      },
    }, { highWaterMark });

    return this[_writable];
  }

  setSignals(signals) {
    webidl.assertBranded(this, SerialPortPrototype);
    
    ops.op_webserial_set_signals(this[_rid], {
      dataTerminalReady: signals.dataTerminalReady,
      break: signals.break,
      requestToSend: signals.requestToSend,
    });
    return Promise.resolve();
  }

  getSignals() {
    webidl.assertBranded(this, SerialPortPrototype);

    return Promise.resolve(ops.op_webserial_get_signals(this[_rid]));
  }

  close() {
    webidl.assertBranded(this, SerialPortPrototype);

    if (this[_state] !== "opened") {
      return Promise.reject(new DOMException("Port is not open", "InvalidStateError"));
    }

    let pendingClosePromise;

    if (this[_readable] === null && this[_writable] === null) {
      pendingClosePromise = Promise.resolve();
    } else {
      this[_pendingClosePromise] = new Deferred();
      pendingClosePromise = this[_pendingClosePromise].promise;
    }

    const cancelPromise = this[_readable]
      ? this[_readable].cancel()
      : Promise.resolve();
    const abortPromise = this[_writable]
      ? this[_writable].abort()
      : Promise.resolve();

    this[_state] = "closing";

    return Promise.all([cancelPromise, abortPromise, pendingClosePromise])
      .then(
        () => {
          ops.op_webserial_close(this[_rid]);
          this[_state] = "closed";
          this[_rid] = undefined;
          this[_readFatal] = this[_writeFatal] = false;
          this[_pendingClosePromise] = undefined;
        })
      .catch(
        (reason) => {
          this[_pendingClosePromise] = undefined;
          throw reason;
        },
      );
  }
}

const SerialPortPrototype = SerialPort.prototype;

const serial = webidl.createBranded(Serial);

export { serial, Serial, SerialPort };
