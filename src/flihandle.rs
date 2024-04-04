#![allow(unused)]
use std::{
    ffi::{c_long, CStr},
    sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    time::Duration,
};

use crate::fli_ffi::*;
use cameraunit::{Error, PixelBpp, ROI};

use image::Pixel;
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
pub struct FLIHandle {
    /// The handle to the camera.
    pub dev: flidev_t,
    /// The exposure time.
    pub exp: AtomicU64,
    /// capturing
    pub capturing: AtomicBool,
    /// image ready
    pub ready: AtomicBool,
    /// dark
    pub dark: AtomicBool,
    /// The pixel bit depth.
    pub bpp: AtomicU32,
}


impl Drop for FLIHandle {
    fn drop(&mut self) {
        let handle = self.dev;
        let res = unsafe { FLICancelExposure(handle) };
        if res != 0 {
            warn!("Error cancelling exposure: {}", res);
        }
        if self.set_temperature(35.0).is_err() {
            warn!("Error setting temperature: {}", res);
        }
        unsafe { FLIClose(self.dev) };
    }
}

impl FLIHandle {
    pub fn new(handle: flidev_t) -> Self {
        FLIHandle {
            dev: handle,
            exp: AtomicU64::new(100),
            capturing: AtomicBool::new(false),
            ready: AtomicBool::new(false),
            dark: AtomicBool::new(false),
            bpp: AtomicU32::new(16),
        }
    }

    pub fn image_ready(&self) -> Result<bool, Error> {
        let capturing = self.capturing.load(Ordering::SeqCst);
        if capturing {
            let mut status: c_long = 0;
            let res = unsafe { FLIGetExposureStatus(self.dev, &mut status) };
            if res != 0 {
                self.capturing.store(false, Ordering::SeqCst);
                return Err(Error::ExposureFailed(format!(
                    "Error getting exposure status: {}",
                    res
                )));
            }
            if status == 0 {
                self.ready.store(true, Ordering::SeqCst);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(Error::GeneralError("Not capturing".to_string()))
        }
    }

    pub fn is_capturing(&self) -> Result<bool, Error> {
        let capturing = self.capturing.load(Ordering::SeqCst);
        if !capturing {
            return Ok(false);
        }
        let status: c_long = 0;
        let res = unsafe { FLIGetExposureStatus(self.dev, status as *mut c_long) };
        if res != 0 {
            self.capturing.store(false, Ordering::SeqCst);
            return Err(Error::ExposureFailed(format!(
                "Error getting exposure status: {}",
                res
            )));
        }
        if status > 0 {
            Ok(true)
        } else {
            self.capturing.store(false, Ordering::SeqCst);
            self.ready.store(true, Ordering::SeqCst);
            Ok(false)
        }
    }

    pub fn cancel_capture(&self) -> Result<(), Error> {
        FLICALL!(FLICancelExposure(self.dev));
        self.capturing.store(false, Ordering::SeqCst);
        self.ready.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub fn get_temperature(&self) -> Result<f32, Error> {
        let mut temp: f64 = 0.;
        FLICALL!(FLIGetTemperature(self.dev, &mut temp));
        Ok(temp as f32)
    }

    pub fn set_temperature(&self, temp: f32) -> Result<(), Error> {
        if !(-55.0..=45.).contains(&temp) {
            return Err(Error::InvalidValue(format!(
                "Invalid temperature value {}",
                temp
            )));
        }
        FLICALL!(FLISetTemperature(self.dev, temp as f64));
        Ok(())
    }

    pub fn get_cooler_power(&self) -> Result<f64, Error> {
        let mut power: f64 = 0.;
        FLICALL!(FLIGetCoolerPower(self.dev, &mut power));
        Ok(power)
    }

    pub fn get_model(&self) -> Result<String, Error> {
        let mut model = [0i8; 128];
        FLICALL!(FLIGetModel(self.dev, model.as_mut_ptr(), model.len()));
        let model = unsafe { CStr::from_ptr(model.as_ptr()) };
        Ok(model.to_string_lossy().to_string())
    }

    pub fn set_exposure(&self, time: Duration) -> Result<(), Error> {
        let ctime = time.as_millis() as c_long;
        FLICALL!(FLISetExposureTime(self.dev, ctime));
        self.exp.store(time.as_millis() as u64, Ordering::SeqCst);
        Ok(())
    }

    pub fn get_array_size(&self) -> Result<(i32, i32, i32, i32), Error> {
        let mut ul_x: c_long = 0;
        let mut ul_y: c_long = 0;
        let mut lr_x: c_long = 0;
        let mut lr_y: c_long = 0;
        FLICALL!(FLIGetVisibleArea(
            self.dev, &mut ul_x, &mut ul_y, &mut lr_x, &mut lr_y
        ));
        Ok((ul_x as i32, ul_y as i32, lr_x as i32, lr_y as i32))
    }

    pub fn set_visible_area(&self, roi: &ROI) -> Result<(), Error> {
        let ul_x = roi.x_min as c_long;
        let ul_y = roi.y_min as c_long;
        let lr_x = (roi.x_min + roi.width) as c_long;
        let lr_y = (roi.y_min + roi.height) as c_long;
        FLICALL!(FLISetImageArea(self.dev, ul_x, ul_y, lr_x, lr_y));
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
            self.dev,
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
        FLICALL!(FLISetHBin(self.dev, bin as c_long));
        Ok(())
    }

    pub fn set_vbin(&self, bin: u32) -> Result<(), Error> {
        if !(0..16).contains(&bin) {
            return Err(Error::InvalidValue(format!(
                "Invalid vertical binning value {}",
                bin
            )));
        }
        FLICALL!(FLISetVBin(self.dev, bin as c_long));
        Ok(())
    }

    pub fn get_serial(&self) -> Result<String, Error> {
        let mut serial = [0i8; 128];
        FLICALL!(FLIGetSerialString(
            self.dev,
            serial.as_mut_ptr(),
            serial.len()
        ));
        let serial = unsafe { CStr::from_ptr(serial.as_ptr()) };
        Ok(serial.to_string_lossy().to_string())
    }

    pub fn get_camera_mode(&self) -> Result<(c_long, String), Error> {
        let mut mode = [0i8; 128];
        let mut modec: flimode_t = 0;
        FLICALL!(FLIGetCameraMode(self.dev, &mut modec));
        FLICALL!(FLIGetCameraModeString(
            self.dev,
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
            let res = unsafe { FLIGetCameraModeString(self.dev, i, modes.as_mut_ptr(), modes.len()) };
            if res != 0 {
                break;
            }
            let mode = unsafe { CStr::from_ptr(modes.as_ptr()) };
            mode_list.push(mode.to_string_lossy().to_string());
        }
        mode_list
    }

    pub fn set_camera_mode(&self, mode: flimode_t) -> Result<(), Error> {
        FLICALL!(FLISetCameraMode(self.dev, mode));
        Ok(())
    }

    pub fn get_pixel_size(&self) -> Result<(f64, f64), Error> {
        let mut x: f64 = 0.;
        let mut y: f64 = 0.;
        FLICALL!(FLIGetPixelSize(self.dev, &mut x, &mut y));
        Ok((x, y))
    }

    pub fn set_bpp(&self, bpp: PixelBpp) -> Result<(), Error> {
        let bpp = bpp as c_long;
        FLICALL!(FLISetBitDepth(self.dev, bpp as c_long));
        self.bpp.store(bpp as u32, Ordering::SeqCst);
        Ok(())
    }

    pub fn get_bpp(&self) -> PixelBpp {
        self.bpp.load(Ordering::SeqCst).into()
    }
}
