#![warn(missing_docs)]

use std::{
    ffi::{c_long, CStr},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};

use crate::fli_ffi::*;
use cameraunit::{Error, ROI};

use log::warn;

macro_rules! FLICALL {
    ($func:ident($($arg:expr),*)) => {
        {
            let res = unsafe { $func($($arg),*) };
            if res != 0 {
                return Err(Error::GeneralError(format!("Error calling {}(): {}", stringify!($func), res)));
            }
        }
    };
}

pub const FLIDOMAIN_CAMERA: i64 = (FLIDEVICE_CAMERA | FLIDOMAIN_USB) as i64;

#[derive(Debug)]
pub struct FLIHandle(
    /// The handle to the camera.
    pub flidev_t,
    /// The exposure time.
    pub AtomicU64,
    /// capturing
    pub AtomicBool,
    /// image ready
    pub AtomicBool,
    /// dark
    pub AtomicBool,
);

impl FLIHandle {
    pub fn new(handle: flidev_t) -> Self {
        FLIHandle(
            handle,
            AtomicU64::new(100),
            AtomicBool::new(false),
            AtomicBool::new(false),
            AtomicBool::new(false),
        )
    }

    pub fn image_ready(&self) -> Result<bool, Error> {
        let capturing = self.2.load(Ordering::SeqCst);
        if capturing {
            let mut status: c_long = 0;
            let res = unsafe { FLIGetExposureStatus(self.0, &mut status) };
            if res != 0 {
                self.2.store(false, Ordering::SeqCst);
                return Err(Error::ExposureFailed(format!(
                    "Error getting exposure status: {}",
                    res
                )));
            }
            if status > 0 {
                self.3.store(true, Ordering::SeqCst);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(Error::GeneralError("Not capturing".to_string()))
        }
    }

    pub fn is_capturing(&self) -> Result<bool, Error> {
        let capturing = self.2.load(Ordering::SeqCst);
        if !capturing {
            return Ok(false);
        }
        let status: c_long = 0;
        let res = unsafe { FLIGetExposureStatus(self.0, status as *mut c_long) };
        if res != 0 {
            self.2.store(false, Ordering::SeqCst);
            return Err(Error::ExposureFailed(format!(
                "Error getting exposure status: {}",
                res
            )));
        }
        if status > 0 {
            Ok(true)
        } else {
            self.2.store(false, Ordering::SeqCst);
            self.3.store(true, Ordering::SeqCst);
            Ok(false)
        }
    }

    pub fn cancel_capture(&self) -> Result<(), Error> {
        FLICALL!(FLICancelExposure(self.0));
        self.2.store(false, Ordering::SeqCst);
        self.3.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub fn get_temperature(&self) -> Result<f32, Error> {
        let mut temp: f64 = 0.0;
        FLICALL!(FLIGetTemperature(self.0, &mut temp));
        Ok(temp as f32)
    }

    pub fn set_temperature(&self, temp: f32) -> Result<(), Error> {
        if !(-55.0..=45.0).contains(&temp) {
            return Err(Error::InvalidValue(format!(
                "Invalid temperature value {}",
                temp
            )));
        }
        FLICALL!(FLISetTemperature(self.0, temp as f64));
        Ok(())
    }

    pub fn get_cooler_power(&self) -> Result<f64, Error> {
        let mut power: f64 = 0.0;
        FLICALL!(FLIGetCoolerPower(self.0, &mut power));
        Ok(power)
    }

    pub fn get_model(&self) -> Result<String, Error> {
        let mut model = [0i8; 128];
        FLICALL!(FLIGetModel(self.0, model.as_mut_ptr(), model.len()));
        let model = unsafe { CStr::from_ptr(model.as_ptr()) };
        Ok(model.to_string_lossy().to_string())
    }

    pub fn set_exposure(&self, time: Duration) -> Result<(), Error> {
        let ctime = time.as_millis() as c_long;
        FLICALL!(FLISetExposureTime(self.0, ctime));
        self.1.store(time.as_millis() as u64, Ordering::SeqCst);
        Ok(())
    }

    pub fn get_array_size(&self) -> Result<(i32, i32, i32, i32), Error> {
        let mut ul_x: c_long = 0;
        let mut ul_y: c_long = 0;
        let mut lr_x: c_long = 0;
        let mut lr_y: c_long = 0;
        FLICALL!(FLIGetVisibleArea(
            self.0, &mut ul_x, &mut ul_y, &mut lr_x, &mut lr_y
        ));
        println!(
            "ul_x: {}, ul_y: {}, lr_x: {}, lr_y: {}",
            ul_x, ul_y, lr_x, lr_y
        );
        Ok((ul_x as i32, ul_y as i32, lr_x as i32, lr_y as i32))
    }

    pub fn set_visible_area(&self, roi: &ROI) -> Result<(), Error> {
        let ul_x = roi.x_min as c_long;
        let ul_y = roi.y_min as c_long;
        let lr_x = (roi.x_min + roi.width) as c_long;
        let lr_y = (roi.y_min + roi.height) as c_long;
        FLICALL!(FLISetImageArea(self.0, ul_x, ul_y, lr_x, lr_y));
        Ok(())
    }

    pub fn get_readout_dim(&self) -> Result<ROI, Error> {
        let mut width: c_long = 0;
        let mut hoffset: c_long = 0;
        let mut hbin: c_long = 0;
        let mut height: c_long = 0;
        let mut voffset: c_long = 0;
        let mut vbin: c_long = 0;
        FLICALL!(FLIGetReadoutDimensions(
            self.0,
            &mut width,
            &mut hoffset,
            &mut hbin,
            &mut height,
            &mut voffset,
            &mut vbin
        ));
        let width = width as u32;
        let height = height as u32;
        Ok(ROI {
            x_min: hoffset as u32,
            y_min: voffset as u32,
            width,
            height,
            bin_x: hbin as u32,
            bin_y: vbin as u32,
        })
    }

    pub fn set_hbin(&self, bin: u32) -> Result<(), Error> {
        if !(0..16).contains(&bin) {
            return Err(Error::InvalidValue(format!(
                "Invalid horizontal binning value {}",
                bin
            )));
        }
        FLICALL!(FLISetHBin(self.0, bin as c_long));
        Ok(())
    }

    pub fn set_vbin(&self, bin: u32) -> Result<(), Error> {
        if !(0..16).contains(&bin) {
            return Err(Error::InvalidValue(format!(
                "Invalid vertical binning value {}",
                bin
            )));
        }
        FLICALL!(FLISetVBin(self.0, bin as c_long));
        Ok(())
    }

    pub fn get_serial(&self) -> Result<String, Error> {
        let mut serial = [0i8; 128];
        FLICALL!(FLIGetSerialString(
            self.0,
            serial.as_mut_ptr(),
            serial.len()
        ));
        let serial = unsafe { CStr::from_ptr(serial.as_ptr()) };
        Ok(serial.to_string_lossy().to_string())
    }

    pub fn get_camera_mode(&self) -> Result<(c_long, String), Error> {
        let mut mode = [0i8; 128];
        let mut modec: flimode_t = 0;
        FLICALL!(FLIGetCameraMode(self.0, &mut modec));
        FLICALL!(FLIGetCameraModeString(
            self.0,
            modec,
            mode.as_mut_ptr(),
            mode.len()
        ));
        let mode = unsafe { CStr::from_ptr(mode.as_ptr()) };
        Ok((modec, mode.to_string_lossy().to_string()))
    }

    pub fn list_camera_modes(&self) -> Vec<String> {
        let mut modes = [0i8; 128];
        let mut mode_list = Vec::new();
        for i in 0..128 {
            let res = unsafe { FLIGetCameraModeString(self.0, i, modes.as_mut_ptr(), modes.len()) };
            if res != 0 {
                break;
            }
            let mode = unsafe { CStr::from_ptr(modes.as_ptr()) };
            mode_list.push(mode.to_string_lossy().to_string());
        }
        mode_list
    }

    pub fn set_camera_mode(&self, mode: flimode_t) -> Result<(), Error> {
        FLICALL!(FLISetCameraMode(self.0, mode));
        Ok(())
    }

    pub fn get_pixel_size(&self) -> Result<(f64, f64), Error> {
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        FLICALL!(FLIGetPixelSize(self.0, &mut x, &mut y));
        Ok((x, y))
    }
}
