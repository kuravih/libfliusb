#![warn(missing_docs)]

use std::{
    collections::HashMap,
    ffi::{c_long, c_uchar, CStr, CString},
    fmt::Display,
    mem::{transmute, MaybeUninit},
    os::raw,
    str,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
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

use crate::flihandle::*;

pub struct CameraUnitFLI {
    handle: Arc<FLIHandle>,
    info: CameraInfoFLI,
    roi: ROI,
    x_min: i32,
    y_min: i32,
    x_max: i32,
    y_max: i32,
}

#[derive(Debug, Clone)]
pub struct CameraInfoFLI {
    handle: Arc<FLIHandle>,
    /// CCD width in pixels.
    width: u32,
    /// CCD height in pixels.
    height: u32,
    /// Camera name.
    name: String,
    /// Serial
    serial: String,
}

impl Drop for FLIHandle {
    fn drop(&mut self) {
        let handle = self.0;
        let res = unsafe { FLICancelExposure(handle) };
        if res != 0 {
            warn!("Error cancelling exposure: {}", res);
        }
        unsafe { FLIClose(self.0) };
    }
}

/// Get a list of camera IDs.
pub fn get_camera_ids() -> Result<Vec<String>, Error> {
    let mut ptr: *mut *mut i8 = std::ptr::null_mut();
    FLICALL!(FLIList(FLIDOMAIN_CAMERA, &mut ptr));
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
    let cname = CString::new(cname[0])
        .map_err(|_| Error::InvalidFormat("Invalid camera name.".to_string()))?;
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
    let handle = Arc::new(FLIHandle::new(handle));
    let serial = handle.get_serial()?;
    let (x_min, y_min, x_max, y_max) = handle.get_array_size()?;

    let info = CameraInfoFLI {
        handle: handle.clone(),
        width: (x_max - x_min) as u32,
        height: (y_max - y_min) as u32,
        name: name.to_string(),
        serial,
    };

    Ok(CameraUnitFLI {
        handle,
        info,
        x_min,
        y_min,
        x_max,
        y_max,
        roi: ROI {
            x_min: 0,
            y_min: 0,
            width: (x_max - x_min) as u32,
            height: (y_max - y_min) as u32,
            bin_x: 1,
            bin_y: 1,
        },
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

impl CameraInfo for CameraInfoFLI {
    fn camera_ready(&self) -> bool {
        true
    }

    fn camera_name(&self) -> &str {
        &self.name
    }

    fn cancel_capture(&self) -> Result<(), Error> {
        self.handle.cancel_capture()
    }

    fn is_capturing(&self) -> bool {
        self.handle.is_capturing().unwrap_or(false)
    }

    fn get_ccd_width(&self) -> u32 {
        self.width
    }

    fn get_ccd_height(&self) -> u32 {
        self.height
    }

    fn get_uuid(&self) -> Option<&str> {
        Some(&self.serial)
    }

    fn get_temperature(&self) -> Option<f32> {
        self.handle.get_temperature().ok().map(|x| x as f32)
    }

    fn set_temperature(&self, temp: f32) -> Result<f32, Error> {
        self.handle.set_temperature(temp)?;
        Ok(temp)
    }

    fn get_cooler_power(&self) -> Option<f32> {
        self.handle.get_cooler_power().ok().map(|x| x as f32)
    }

    fn get_pixel_size(&self) -> Option<f32> {
        let (x, _) = self.handle.get_pixel_size().ok()?;
        Some(x as f32)
    }
}

impl CameraInfo for CameraUnitFLI {
    fn camera_ready(&self) -> bool {
        true
    }

    fn camera_name(&self) -> &str {
        self.info.camera_name()
    }

    fn cancel_capture(&self) -> Result<(), Error> {
        self.info.cancel_capture()
    }

    fn is_capturing(&self) -> bool {
        self.info.is_capturing()
    }

    fn get_ccd_width(&self) -> u32 {
        self.info.height
    }

    fn get_ccd_height(&self) -> u32 {
        self.info.height
    }

    fn get_uuid(&self) -> Option<&str> {
        self.info.get_uuid()
    }

    fn set_temperature(&self, temperature: f32) -> Result<f32, Error> {
        self.info.set_temperature(temperature)
    }

    fn get_temperature(&self) -> Option<f32> {
        self.info.get_temperature()
    }

    fn get_cooler_power(&self) -> Option<f32> {
        self.info.get_cooler_power()
    }

    fn get_pixel_size(&self) -> Option<f32> {
        self.info.get_pixel_size()
    }
}

impl CameraUnit for CameraUnitFLI {
    fn get_handle(&self) -> Option<&dyn std::any::Any> {
        Some(&self.handle.0)
    }

    fn get_min_exposure(&self) -> Result<Duration, Error> {
        Ok(Duration::from_millis(1))
    }

    fn get_max_exposure(&self) -> Result<Duration, Error> {
        Ok(Duration::from_secs(3600))
    }

    fn set_shutter_open(&mut self, open: bool) -> Result<bool, Error> {
        if self.info.is_capturing() {
            Err(Error::ExposureInProgress)
        } else {
            FLICALL!(FLISetFrameType(self.handle.0, if open { FLI_FRAME_TYPE_NORMAL as i64 } else { FLI_FRAME_TYPE_DARK as i64 }));
            self.handle.4.store(open, Ordering::SeqCst);
            Ok(open)
        }
    }

    fn get_shutter_open(&self) -> Result<bool, Error> {
        Ok(self.handle.4.load(Ordering::SeqCst))
    }

    fn set_flip(&mut self, _x: bool, _y: bool) -> Result<(), Error> {
        Err(Error::Message("Not implemented".to_string()))
    }

    fn get_flip(&self) -> (bool, bool) {
        (false, false)
    }

    fn get_bin_x(&self) -> u32 {
        self.roi.bin_x
    }

    fn get_bin_y(&self) -> u32 {
        self.roi.bin_y
    }

    fn get_status(&self) -> String {
        "Not implemented".to_string()
    }

    fn get_vendor(&self) -> &str {
        "FLI"
    }

    fn capture_image(&self) -> Result<DynamicSerialImage, Error> {
        todo!()
    }

    fn start_exposure(&self) -> Result<(), Error> {
        if !self.handle.2.load(Ordering::SeqCst) {
            self.handle.2.store(true, Ordering::SeqCst);
            FLICALL!(FLIExposeFrame(self.handle.0));
            self.handle.3.store(false, Ordering::SeqCst);
            Ok(())
        } else {
            Err(Error::ExposureInProgress)
        }
    }

    fn download_image(&self) -> Result<DynamicSerialImage, Error> {
        todo!()
    }

    fn image_ready(&self) -> Result<bool, Error> {
        self.handle.image_ready()
    }

    fn set_exposure(&mut self, exposure: Duration) -> Result<Duration, Error> {
        if self.handle.2.load(Ordering::SeqCst) {
            return Err(Error::ExposureInProgress);
        }
        if exposure < self.get_min_exposure()? || exposure > self.get_max_exposure()? {
            return Err(Error::InvalidValue(format!(
                "Invalid exposure time: {}",
                exposure.as_millis()
            )));
        }
        self.handle.set_exposure(exposure)?;
        Ok(exposure)
    }

    fn get_exposure(&self) -> Duration {
        Duration::from_millis(self.handle.1.load(Ordering::SeqCst))
    }

    fn set_roi(&mut self, roi: &ROI) -> Result<&ROI, Error> {
        if self.info.is_capturing() {
            Err(Error::ExposureInProgress)
        }
        else {
            if roi.width == 0 || roi.height == 0 {
                return Err(Error::InvalidValue("Invalid ROI".to_string()));
            }

            if roi.width * roi.bin_x > self.info.width || roi.height * roi.bin_y > self.info.height {
                return Err(Error::InvalidValue("Invalid ROI".to_string()));
            }

            let x_min = (roi.x_min * self.roi.bin_x) as i64;
            let y_min = (roi.y_min * self.roi.bin_y) as i64;

            let ul_x = self.x_min as i64 + x_min;
            let ul_y = self.y_min as i64 + y_min;

            if ul_x < self.x_min.into() || ul_x >= self.x_max.into() || ul_y < self.y_min.into() || ul_y >= self.y_max.into() {
                return Err(Error::InvalidValue("Invalid ROI".to_string()));
            }

            let lr_x = ul_x + roi.width as i64;
            let lr_y = ul_y + roi.height as i64;

            FLICALL!(FLISetImageArea(self.handle.0, ul_x, ul_y, lr_x, lr_y));
            FLICALL!(FLISetHBin(self.handle.0, roi.bin_x as c_long));
            FLICALL!(FLISetVBin(self.handle.0, roi.bin_y as c_long));

            self.roi = self.handle.get_readout_dim()?;
            Ok(&self.roi)
        }
    }

    fn get_roi(&self) -> &ROI {
        &self.roi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_temperature() {
        let cam = open_first_camera().unwrap();
        let temp = cam.handle.get_temperature().unwrap();
        println!("Temperature: {}", temp);
        assert!(temp >= -100.0);
    }

    #[test]
    fn test_set_temperature() {
        let cam = open_first_camera().unwrap();
        cam.handle.set_temperature(-20.0).unwrap();
    }

    #[test]
    fn test_get_cooler_power() {
        let cam = open_first_camera().unwrap();
        let power = cam.handle.get_cooler_power().unwrap();
        println!("Cooler power: {}", power);
        assert!(power >= 0.0);
    }

    #[test]
    fn test_get_model() {
        let cam = open_first_camera().unwrap();
        let model = cam.handle.get_model().unwrap();
        println!("Model: {}", model);
        assert!(!model.is_empty());
    }

    #[test]
    fn test_set_exposure() {
        let cam = open_first_camera().unwrap();
        cam.handle.set_exposure(Duration::from_millis(100)).unwrap();
    }

    #[test]
    fn test_get_array_size() {
        let cam = open_first_camera().unwrap();
        let size = cam.handle.get_array_size().unwrap();
        println!("Array size: {:?}", size);
        assert!(size.0 > 0 && size.1 > 0);
    }

    #[test]
    fn test_set_visible_area() {
        let cam = open_first_camera().unwrap();
        println!("{:?}", cam.handle.get_array_size().unwrap());
        let roi = ROI {
            x_min: 100,
            y_min: 100,
            width: 100,
            height: 100,
            bin_x: 1,
            bin_y: 1,
        };
        cam.handle.set_visible_area(&roi).unwrap();
        println!("{:?}", cam.handle.get_array_size().unwrap());
        println!("{}", cam.handle.get_readout_dim().unwrap());
        cam.handle.set_hbin(2).unwrap();
        cam.handle.set_vbin(2).unwrap();
        println!("{}", cam.handle.get_readout_dim().unwrap());
    }

    #[test]
    fn test_get_serial() {
        let cam = open_first_camera().unwrap();
        let serial = cam.handle.get_serial().unwrap();
        println!("Serial: {}", serial);
        assert!(!serial.is_empty());
    }

    #[test]
    fn test_get_camera_mode() {
        let cam = open_first_camera().unwrap();
        let mode = cam.handle.list_camera_modes();
        println!("Camera modes: {:?}", mode);
    }
}
