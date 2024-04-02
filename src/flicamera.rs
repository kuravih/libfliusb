#![warn(missing_docs)]

use std::{
    collections::HashMap,
    ffi::{c_long, c_uchar, CStr, CString},
    fmt::Display,
    mem::{transmute, MaybeUninit},
    os::raw,
    str,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::fli_ffi::*;

use cameraunit::{
    CameraInfo, CameraUnit, DynamicSerialImage, Error, ImageMetaData, SerialImageBuffer, ROI,
};
use log::{info, warn};

const FLIDOMAIN_CAMERA: i64 = (FLIDEVICE_CAMERA | FLIDOMAIN_USB) as i64;

#[derive(Debug)]
struct FLIHandle(Arc<flidev_t>);

#[derive(Debug)]
pub struct CameraUnitFLI {
    handle: FLIHandle,
}

impl Drop for FLIHandle {
    fn drop(&mut self) {
        let handle = *self.0;
        let res = unsafe { FLICancelExposure(handle) };
        if res != 0 {
            warn!("Error cancelling exposure: {}", res);
        }
        unsafe { FLIClose(*self.0) };
    }
}

/// Get a list of camera IDs.
pub fn get_camera_ids() -> Result<Vec<String>, Error> {
    let mut ptr: *mut *mut i8 = std::ptr::null_mut();
    let res = unsafe { FLIList(FLIDOMAIN_CAMERA, &mut ptr) };
    if res != 0 {
        warn!("Error getting number of cameras: {}", res);
    }
    let mut i = 0;
    let mut out = Vec::new();
    while !ptr.is_null() {
        let mptr = unsafe { *ptr.offset(i) };
        if mptr.is_null() {
            break;
        }
        let cstr = unsafe { CStr::from_ptr(*ptr.offset(i)) };
        out.push(
            cstr.to_str()
                .map_err(|_| {
                    Error::InvalidFormat(format!(
                        "Error converting camera ID {:?} to string.",
                        cstr
                    ))
                })?
                .to_string(),
        );
        if cstr.to_str().is_ok() {}
        i += 1;
    }
    unsafe { FLIFreeList(ptr) };
    Ok(out)
}

/// Get the number of cameras connected to the system.
pub fn num_cameras() -> i32 {
    let camlist = get_camera_ids();
    match camlist {
        Ok(list) => list.len() as i32,
        Err(_) => 0,
    }
}

/// Open a camera by name.
pub fn open_camera(name: &str) -> Result<CameraUnitFLI, Error> {
    let mut handle: flidev_t = FLI_INVALID_DEVICE.into();
    let cname: Vec<&str> = name.split(';').collect();
    let cname =
        CString::new(cname[0]).map_err(|_| Error::InvalidFormat("Invalid camera name.".to_string()))?;
    let ptr = cname.into_raw();
    let res = unsafe { FLIOpen(&mut handle, ptr, FLIDOMAIN_CAMERA) };
    let _ = unsafe { CString::from_raw(ptr) };
    if res != 0 {
        return Err(Error::GeneralError(format!(
            "Error opening camera {} ({}): {}",
            name, handle, res
        )));
    }
    if handle == FLI_INVALID_DEVICE.into() {
        return Err(Error::NoCamerasAvailable);
    }
    Ok(CameraUnitFLI {
        handle: FLIHandle(Arc::new(handle)),
    })
}

/// Open the first available camera.
pub fn open_first_camera() -> Result<CameraUnitFLI, Error> {
    let camlist = get_camera_ids()?;
    if camlist.is_empty() {
        return Err(Error::NoCamerasAvailable);
    }
    open_camera(&camlist[0])
}
