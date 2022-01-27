use crate::error::Error;
use std::ffi::CStr;
use std::ffi::OsStr;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::os::raw::c_void;
use std::os::unix::ffi::OsStrExt;
use std::ptr::null_mut;

pub type Handle = *mut c_void;

#[inline]
pub unsafe fn dlopen(name: &OsStr, flags: u32) -> Result<Handle, Error> {
  let mut v: Vec<u8> = Vec::new();
  let cstr = if !name.is_empty() && name.as_bytes()[name.len() - 1] == 0 {
    CStr::from_bytes_with_nul_unchecked(name.as_bytes())
  } else {
    v.extend_from_slice(name.as_bytes());
    v.push(0);
    CStr::from_bytes_with_nul_unchecked(v.as_slice())
  };
  let handle = libc::dlopen(cstr.as_ptr(), flags as libc::c_int);
  if handle.is_null() {
    Err(Error::OpenLibrary(IoError::new(
      ErrorKind::Other,
      CStr::from_ptr(libc::dlerror())
        .to_string_lossy()
        .to_string(),
    )))
  } else {
    Ok(handle)
  }
}

#[inline]
pub unsafe fn dlsym(handle: Handle, name: &CStr) -> Result<*mut (), Error> {
  let _ = libc::dlerror();
  let symbol = libc::dlsym(handle, name.as_ptr());
  if symbol.is_null() {
    let msg = libc::dlerror();
    if !msg.is_null() {
      return Err(Error::GetSymbol(IoError::new(
        ErrorKind::Other,
        CStr::from_ptr(msg).to_string_lossy().to_string(),
      )));
    }
  }
  Ok(symbol as *mut ())
}

#[inline]
pub fn dlclose(handle: Handle) -> Handle {
  unsafe { libc::dlclose(handle) };
  null_mut()
}
