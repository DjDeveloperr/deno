use crate::error::Error;
use std::ffi::CStr;
use std::ffi::OsStr;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi::shared::minwindef::HMODULE;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::FreeLibrary;
use winapi::um::libloaderapi::GetProcAddress;
use winapi::um::libloaderapi::LoadLibraryExW;

pub type Handle = HMODULE;

unsafe fn get_win_error() -> IoError {
  let error = GetLastError();
  if error == 0 {
    IoError::new(
      ErrorKind::Other,
      "Could not obtain information about the error",
    )
  } else {
    IoError::from_raw_os_error(error as i32)
  }
}

#[inline]
pub unsafe fn dlopen(name: &OsStr, flags: u32) -> Result<Handle, Error> {
  let wide_name: Vec<u16> = name.encode_wide().chain(Some(0)).collect();
  let handle = LoadLibraryExW(wide_name.as_ptr(), null_mut(), flags);
  if handle.is_null() {
    Err(Error::OpenLibrary(get_win_error()))
  } else {
    Ok(handle)
  }
}

#[inline]
pub unsafe fn dlsym(handle: Handle, name: &CStr) -> Result<*mut (), Error> {
  let symbol = GetProcAddress(handle, name.as_ptr());
  if symbol.is_null() {
    Err(Error::GetSymbol(get_win_error()))
  } else {
    Ok(symbol as *mut ())
  }
}

#[inline]
pub fn dlclose(handle: Handle) -> Handle {
  unsafe { FreeLibrary(handle) };
  null_mut()
}
