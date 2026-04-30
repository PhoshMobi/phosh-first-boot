// SPDX-FileCopyrightText: 2026 Phosh.mobi e.V.
// SPDX-License-Identifier: GPL-3.0-or-later

use libc::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::ptr;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum CryptErrorKind {
    PreferredMethodFailed,
    SaltGenFailed,
    HashFailed,
    StringAllocFailed,
    StringConversionError,
}

#[derive(Debug)]
pub struct CryptError {
    kind: CryptErrorKind,
    detail: Option<String>,
}

impl CryptError {
    fn fmt_error(&self) -> String {
        let msg: String = match self.kind {
            CryptErrorKind::PreferredMethodFailed => "crypt_preferred_method failed".into(),
            CryptErrorKind::SaltGenFailed => "crypt_gensalt_ra failed".into(),
            CryptErrorKind::HashFailed => "crypt_ra failed".into(),
            CryptErrorKind::StringAllocFailed => "Failed to allocate C string".into(),
            CryptErrorKind::StringConversionError => "Failed to convert to String".into(),
        };

        if let Some(detail) = &self.detail {
            return msg + format!(": {}", detail).as_str();
        };

        msg
    }
}

impl fmt::Display for CryptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fmt_error())
    }
}

impl Error for CryptError {}

pub struct Crypt;

impl Crypt {
    pub fn hash(password: &str) -> Result<String, CryptError> {
        unsafe { hash_internal(password) }
    }
}

// FFI unsafe bits

extern "C" {
    fn crypt_preferred_method() -> *const c_char;

    fn crypt_gensalt_ra(
        prefix: *const c_char,
        count: libc::c_ulong,
        rbytes: *mut *mut c_void,
        rsize: *mut c_int,
    ) -> *mut c_char;

    fn crypt_ra(
        key: *const c_char,
        setting: *const c_char,
        data: *mut *mut c_void,
        size: *mut c_int,
    ) -> *mut c_char;
}

unsafe fn hash_internal(password: &str) -> Result<String, CryptError> {
    let method_ptr = crypt_preferred_method();
    if method_ptr.is_null() {
        return Err(CryptError {
            kind: CryptErrorKind::PreferredMethodFailed,
            detail: Some(format!("{}", std::io::Error::last_os_error())),
        });
    }
    let method = CStr::from_ptr(method_ptr);

    let salt_ptr = crypt_gensalt_ra(method.as_ptr(), 0, ptr::null_mut(), ptr::null_mut());
    if salt_ptr.is_null() {
        return Err(CryptError {
            kind: CryptErrorKind::SaltGenFailed,
            detail: Some(format!("{}", std::io::Error::last_os_error())),
        });
    }

    let Ok(salt) = CStr::from_ptr(salt_ptr).to_str() else {
        return Err(CryptError {
            kind: CryptErrorKind::StringConversionError,
            detail: Some("Salt is not valid".into()),
        });
    };

    let password_c = CString::new(password).map_err(|_| CryptError {
        kind: CryptErrorKind::StringAllocFailed,
        detail: None,
    })?;

    let mut data: *mut c_void = ptr::null_mut();
    let mut size: c_int = 0;
    let hash_ptr = crypt_ra(
        password_c.as_ptr(),
        salt.as_ptr() as *const c_char,
        &mut data,
        &mut size,
    );
    if hash_ptr.is_null() {
        return Err(CryptError {
            kind: CryptErrorKind::HashFailed,
            detail: Some(format!("{}", std::io::Error::last_os_error())),
        });
    }

    let Ok(hash) = CStr::from_ptr(hash_ptr).to_str() else {
        return Err(CryptError {
            kind: CryptErrorKind::StringConversionError,
            detail: Some("Password is not valid".into()),
        });
    };
    let hash = hash.to_string();

    libc::free(data);

    Ok(hash)
}
