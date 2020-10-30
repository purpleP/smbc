#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use libc::{c_char, c_int, c_void, strncpy};
use std::borrow::Cow;
use std::ffi::{CStr, FromBytesWithNulError, NulError};
use std::fmt;
use std::io;
use std::io::Read;

#[derive(Debug)]
pub enum Error {
    InvalidPath,
    ConfigError,
    OutOfMemory,
    UnexpectedSystemError(Option<i32>),
}

impl From<FromBytesWithNulError> for Error {
    fn from(_err: FromBytesWithNulError) -> Self {
        Self::InvalidPath
    }
}

impl From<NulError> for Error {
    fn from(_err: NulError) -> Self {
        Self::InvalidPath
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for Error {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath => {
                write!(f, "Path is not a valid path or cstring")
            }
            Self::ConfigError => {
                write!(f, "Could not read smb.conf")
            }
            Self::OutOfMemory => {
                write!(f, "Not enough memory")
            }
            Self::UnexpectedSystemError(Some(code)) => {
                write!(f, "Unexpected system error with error code {}", code)
            }
            Self::UnexpectedSystemError(None) => {
                write!(f, "Unexpected system error")
            }
        }
    }
}

pub struct Context<'a> {
    ctx: *mut SMBCCTX,
    auth: &'a dyn for<'b> Fn(
        &'b [u8],
        &'b [u8],
    )
        -> (Cow<'a, [u8]>, Cow<'a, [u8]>, Cow<'a, [u8]>),
}

pub struct File {
    ctx: *mut SMBCCTX,
    file: *mut SMBCFILE,
}

const DEFAULT_AUTH: (
    Cow<'static, [u8]>,
    Cow<'static, [u8]>,
    Cow<'static, [u8]>,
) = (
    Cow::Borrowed(b"WORKGROUP\0"),
    Cow::Borrowed(b"guest\0"),
    Cow::Borrowed(b"\0"),
);

impl<'a> Context<'a> {
    pub fn new<F>(auth: &'a F) -> Result<Context<'a>, Error>
    where
        F: for<'b> Fn(
            &'b [u8],
            &'b [u8],
        ) -> (Cow<'a, [u8]>, Cow<'a, [u8]>, Cow<'a, [u8]>),
    {
        unsafe {
            let ctx = smbc_new_context();
            let ctx = smbc_init_context(ctx);
            if ctx.is_null() {
                match io::Error::last_os_error().raw_os_error() {
                    Some(libc::ENOMEM) => return Err(Error::OutOfMemory),
                    e => return Err(Error::UnexpectedSystemError(e)),
                }
            }
            smbc_setOptionUserData(ctx, auth as *const _ as *mut c_void);
            smbc_setFunctionAuthDataWithContext(
                ctx,
                Some(Self::auth_internal::<F>),
            );
            if ctx.is_null() {
                match io::Error::last_os_error().raw_os_error() {
                    Some(libc::ENOMEM) => return Err(Error::OutOfMemory),
                    Some(libc::ENOENT) => return Err(Error::ConfigError),
                    e => return Err(Error::UnexpectedSystemError(e)),
                }
            }
            Ok(Context { ctx, auth })
        }
    }

    extern "C" fn auth_internal<F: 'a>(
        ctx: *mut SMBCCTX,
        srv: *const c_char,
        shr: *const c_char,
        wg: *mut c_char,
        _wglen: c_int,
        un: *mut c_char,
        _unlen: c_int,
        pw: *mut c_char,
        _pwlen: c_int,
    ) where
        F: for<'b> Fn(
            &'b [u8],
            &'b [u8],
        ) -> (Cow<'a, [u8]>, Cow<'a, [u8]>, Cow<'a, [u8]>),
    {
        unsafe {
            let srv = CStr::from_ptr(srv).to_bytes_with_nul();
            let shr = CStr::from_ptr(shr).to_bytes_with_nul();
            let auth: &'a F = std::mem::transmute(
                smbc_getOptionUserData(ctx) as *const c_void
            );
            let auth = std::panic::AssertUnwindSafe(auth);
            let (raw_workgroup, raw_username, raw_password) =
                std::panic::catch_unwind(|| auth(srv, shr))
                    .unwrap_or(DEFAULT_AUTH);
            let workgroup = CStr::from_bytes_with_nul_unchecked(&raw_workgroup);
            let username = CStr::from_bytes_with_nul_unchecked(&raw_username);
            let password = CStr::from_bytes_with_nul_unchecked(&raw_password);
            strncpy(wg, workgroup.as_ptr(), raw_workgroup.len());
            strncpy(un, username.as_ptr(), raw_username.len());
            strncpy(pw, password.as_ptr(), raw_password.len());
        }
    }

    pub fn open_ro<P: AsRef<[u8]>>(&self, path: P) -> Result<File, Error> {
        unsafe {
            let open_fn = smbc_getFunctionOpen(self.ctx).unwrap();
            let path = CStr::from_bytes_with_nul(path.as_ref())?;
            let file = open_fn(self.ctx, path.as_ptr(), libc::O_RDONLY, 0o644);
            Ok(File {
                ctx: self.ctx,
                file,
            })
        }
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let read_fn = smbc_getFunctionRead(self.ctx)
                .ok_or(io::Error::from_raw_os_error(libc::EINVAL))?;
            let n = read_fn(
                self.ctx,
                self.file,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as _,
            );
            if n < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(n as usize)
            }
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            if let Some(close_fn) = smbc_getFunctionClose(self.ctx) {
                close_fn(self.ctx, self.file);
            }
        }
    }
}

impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        unsafe {
            smbc_free_context(self.ctx, 1);
        }
    }
}
