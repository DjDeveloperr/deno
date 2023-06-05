// Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.

use deno_core::AsyncRefCell;
use deno_core::ResourceId;
use deno_core::ZeroCopyBuf;
use deno_core::error::AnyError;
use deno_core::op;
use deno_core::OpState;
use deno_core::Resource;
use deno_core::RcRef;
use tokio::io::ReadHalf;
use tokio::io::WriteHalf;

use std::borrow::Cow;
use std::rc::Rc;
use std::cell::RefCell;
use serde::Serialize;
use tokio_serial::available_ports;
use tokio_serial::new;
use tokio_serial::SerialStream;
use tokio_serial::SerialPortType;
use tokio_serial::SerialPortBuilderExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::split;

deno_core::extension!(deno_webserial,
  deps = [ deno_webidl ],
  ops = [
    op_webserial_get_ports,
    op_webserial_open,
    op_webserial_close,
    op_webserial_read,
    op_webserial_write,
  ],
  esm = [ "01_webserial.js" ],
  options = {},
  state = |state, options| {},
);

pub struct SerialPortResource {
  pub r: AsyncRefCell<ReadHalf<SerialStream>>,
  pub w: AsyncRefCell<WriteHalf<SerialStream>>,
}

impl Resource for SerialPortResource {
  fn name(&self) -> Cow<str> {
    "fetchRequest".into()
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialPortInfo {
  name: String,
  usb_product_name: Option<String>,
  usb_manufacturer: Option<String>,
  usb_vendor_id: Option<u16>,
  usb_product_id: Option<u16>,
}

#[op]
pub fn op_webserial_get_ports(
  _state: &mut OpState,
  _persistent: bool,
) -> Result<Vec<SerialPortInfo>, AnyError> {
  let ports = available_ports()?;

  return Ok(ports
    .into_iter()
    .map(|port| {
      SerialPortInfo {
        name: port.port_name,
        usb_product_name: match &port.port_type {
          SerialPortType::UsbPort(info) => info.product.clone(),
          _ => None,
        },
        usb_manufacturer: match &port.port_type {
          SerialPortType::UsbPort(info) => info.manufacturer.clone(),
          _ => None,
        },
        usb_vendor_id: match &port.port_type {
          SerialPortType::UsbPort(info) => Some(info.vid),
          _ => None,
        },
        usb_product_id: match &port.port_type {
          SerialPortType::UsbPort(info) => Some(info.pid),
          _ => None,
        },
      }
    })
    .collect());
}

#[op]
pub fn op_webserial_open(
  state: &mut OpState,
  path: String,
  baud_rate: u32,
  data_bits: u8,
  stop_bits: u8,
  parity: String,
  flow_control: String,
) -> Result<ResourceId, AnyError> {
  let builder = new(path, baud_rate)
    .data_bits(match data_bits {
      7 => tokio_serial::DataBits::Seven,
      8 => tokio_serial::DataBits::Eight,
      _ => unreachable!(),
    })
    .stop_bits(match stop_bits {
      1 => tokio_serial::StopBits::One,
      2 => tokio_serial::StopBits::Two,
      _ => unreachable!(),
    })
    .parity(match parity.as_str() {
      "none" => tokio_serial::Parity::None,
      "even" => tokio_serial::Parity::Even,
      "odd" => tokio_serial::Parity::Odd,
      _ => unreachable!(),
    })
    .flow_control(match flow_control.as_str() {
      "none" => tokio_serial::FlowControl::None,
      "software" => tokio_serial::FlowControl::Software,
      "hardware" => tokio_serial::FlowControl::Hardware,
      _ => unreachable!(),
    });

  let stream = builder.open_native_async()?;

  let (reader, writer) = split(stream);

  let resource = SerialPortResource {
    r: AsyncRefCell::new(reader),
    w: AsyncRefCell::new(writer),
  };

  let rid = state.resource_table.add(resource);

  Ok(rid)
}

#[op(fast)]
pub fn op_webserial_close(
  state: &mut OpState,
  rid: ResourceId,
) -> Result<(), AnyError> {
  state.resource_table.close(rid)?;
  Ok(())
}

#[op(fast)]
pub async fn op_webserial_read(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  mut buffer: ZeroCopyBuf,
) -> Result<usize, AnyError> {
  let mut stream = {
    let state = state.borrow_mut();
    let resource = state.resource_table.get::<SerialPortResource>(rid)?;
    let resource = RcRef::map(&resource, |r| &r.r);
    resource.borrow_mut().await
  };
  let n_read = stream.read(&mut buffer[..]).await?;
  Ok(n_read)
}

#[op(fast)]
pub async fn op_webserial_write(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  buffer: ZeroCopyBuf,
) -> Result<(), AnyError> {
  let mut stream = {
    let state = state.borrow_mut();
    let resource = state.resource_table.get::<SerialPortResource>(rid)?;
    let resource = RcRef::map(&resource, |r| &r.w);
    resource.borrow_mut().await
  };
  stream.write_all(&buffer[..]).await?;
  Ok(())
}
