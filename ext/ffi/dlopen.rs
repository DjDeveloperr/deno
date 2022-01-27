use crate::error::Error;
use std::ffi::CString;
use std::ffi::OsStr;

#[cfg(unix)]
use crate::unix as raw;
#[cfg(windows)]
use crate::windows as raw;

use std::mem::transmute_copy;
use std::os::raw::c_void;

#[derive(Debug)]
pub struct Library(raw::Handle);

impl Library {
  pub fn open<S>(name: S, mut flags: u32) -> Result<Library, Error>
  where
    S: AsRef<OsStr>,
  {
    #[cfg(unix)]
    if flags == 0 {
      flags = (libc::RTLD_LAZY | libc::RTLD_LOCAL) as u32;
    }

    let handle = unsafe { raw::dlopen(name.as_ref(), flags) }?;
    Ok(Self(handle))
  }

  pub fn symbol(&self, name: &str) -> Result<*const c_void, Error> {
    let name = CString::new(name)?;
    let raw = unsafe { raw::dlsym(self.0, &name)? };
    if raw.is_null() {
      Err(Error::NullSymbol)
    } else {
      let ptr: *const c_void = unsafe { transmute_copy(&raw) };
      Ok(ptr)
    }
  }
}

impl Drop for Library {
  fn drop(&mut self) {
    self.0 = raw::dlclose(self.0);
  }
}

unsafe impl Sync for Library {}
unsafe impl Send for Library {}
