#![warn(missing_docs)]

use std::{
    collections::HashMap,
    ffi::{c_long, c_uchar, CStr},
    fmt::Display,
    mem::MaybeUninit,
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

pub struct CameraUnitFLI {
    handle: Arc<flidev_t>,
}

/// Get the number of cameras connected to the system.
pub fn num_cameras() -> i32 {
    let mut ptr: *mut *mut i8 = std::ptr::null_mut();
    let res = unsafe { FLIList(FLIDEVICE_CAMERA as i64, &mut ptr) };
    if res != 0 {
        warn!("Error getting number of cameras: {}", res);
    }
    let mut count = 0;
    let mut i = 0;
    while !ptr.is_null() {
        let cstr = unsafe { CStr::from_ptr(*ptr.offset(i)) };
        if cstr.to_str().is_ok() {
            count += 1;
        }
        i += 1;
    }
    unsafe { FLIFreeList(ptr) };
    count
}

pub fn open_first_camera() -> Result<CameraUnitFLI, Error> {
    let mut ptr: *mut *mut i8 = std::ptr::null_mut();
    let res = unsafe { FLIList(FLIDEVICE_CAMERA as i64, &mut ptr) };
    if res != 0 {
        return Err(Error::GeneralError(format!("Error getting camera list: {}", res)));
    }
    let mut i = 0;
    let mut handle: flidev_t = FLI_INVALID_DEVICE.into();
    while !ptr.is_null() {
        let cstr = unsafe { CStr::from_ptr(*ptr.offset(i)) };
        if cstr.to_str().is_ok() {
            let res = unsafe { FLIOpen(&mut handle, *ptr.offset(i), FLIDEVICE_CAMERA as i64) };
            if res == 0 {
                break;
            }
        }
        i += 1;
    }
    unsafe { FLIFreeList(ptr) };
    if handle == FLI_INVALID_DEVICE.into() {
        return Err(Error::NoCamerasAvailable);
    }
    Ok(CameraUnitFLI { 
        handle: Arc::new(handle),
    })
}