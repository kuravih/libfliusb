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

const FLIDOMAIN_CAMERA: i64 = (FLIDEVICE_CAMERA | FLIDOMAIN_USB) as i64;

#[derive(Debug, Clone)]
struct FLIHandle(
    Arc<flidev_t>, // The handle to the camera.
    Duration,      // The exposure time.
);

pub struct CameraUnitFLI {
    handle: FLIHandle,
    info: CameraInfoFLI,
    roi: ROI,
    x_min: i32,
    y_min: i32,
    x_max: i32,
    y_max: i32,
}

#[derive(Debug, Clone)]
pub struct CameraInfoFLI {
    handle: FLIHandle,
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
    let handle = FLIHandle(Arc::new(handle), Duration::from_millis(100));
    let serial = get_serial(&handle)?;
    let (x_min, y_min, x_max, y_max) = get_array_size(&handle)?;

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
        FLICALL!(FLICancelExposure(*self.handle.0));
        Ok(())
    }

    fn is_capturing(&self) -> bool {
        let mut status: c_long = 0;
        if unsafe { FLIGetExposureStatus(*self.handle.0, &mut status) } != 0 {
            return false;
        }
        status > 0
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
        get_temperature(&self.handle).ok().map(|x| x as f32)
    }

    fn set_temperature(&self, temp: f32) -> Result<f32, Error> {
        set_temperature(&self.handle, temp as f64)?;
        Ok(temp)
    }

    fn get_cooler_power(&self) -> Option<f32> {
        get_cooler_power(&self.handle).ok().map(|x| x as f32)
    }

    fn get_pixel_size(&self) -> Option<f32> {
        let (x, _) = get_pixel_size(&self.handle).ok()?;
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
        }
        else {
            // FLICALL!(FLISetFrameType(*self.handle.0, if open { FLI_FRAME_TYPE_LIGHT } else { FLI_FRAME_TYPE_DARK }));
            // self.handle.2 = open;    
            Ok(open)
        }
    }

    fn get_shutter_open(&self) -> Result<bool, Error> {
        // Ok(self.handle.2)
        Ok(false)
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
        FLICALL!(FLIExposeFrame(*self.handle.0));
        Ok(())
    }

    fn download_image(&self) -> Result<DynamicSerialImage, Error> {
        todo!()
    }

    fn image_ready(&self) -> Result<bool, Error> {
        todo!()
    }

    fn set_exposure(&mut self, _exposure: Duration) -> Result<Duration, Error> {
        todo!()
    }

    fn get_exposure(&self) -> Duration {
        todo!()
    }

    fn set_roi(&mut self, roi: &ROI) -> Result<&ROI, Error> {
        todo!()
    }

    fn get_roi(&self) -> &ROI {
        &self.roi
    }
}

fn get_temperature(handle: &FLIHandle) -> Result<f64, Error> {
    let mut temp: f64 = 0.0;
    FLICALL!(FLIGetTemperature(*handle.0, &mut temp));
    Ok(temp)
}

fn set_temperature(handle: &FLIHandle, temp: f64) -> Result<(), Error> {
    if !(-55.0..=45.0).contains(&temp) {
        return Err(Error::InvalidValue(format!(
            "Invalid temperature value {}",
            temp
        )));
    }
    FLICALL!(FLISetTemperature(*handle.0, temp));
    Ok(())
}

fn get_cooler_power(handle: &FLIHandle) -> Result<f64, Error> {
    let mut power: f64 = 0.0;
    FLICALL!(FLIGetCoolerPower(*handle.0, &mut power));
    Ok(power)
}

fn get_model(handle: &FLIHandle) -> Result<String, Error> {
    let mut model = [0i8; 128];
    FLICALL!(FLIGetModel(*handle.0, model.as_mut_ptr(), model.len()));
    let model = unsafe { CStr::from_ptr(model.as_ptr()) };
    Ok(model.to_string_lossy().to_string())
}

fn set_exposure(handle: &mut FLIHandle, time: Duration) -> Result<(), Error> {
    let ctime = time.as_millis() as c_long;
    FLICALL!(FLISetExposureTime(*handle.0, ctime));
    handle.1 = time;
    Ok(())
}

fn get_array_size(handle: &FLIHandle) -> Result<(i32, i32, i32, i32), Error> {
    let mut ul_x: c_long = 0;
    let mut ul_y: c_long = 0;
    let mut lr_x: c_long = 0;
    let mut lr_y: c_long = 0;
    FLICALL!(FLIGetVisibleArea(
        *handle.0, &mut ul_x, &mut ul_y, &mut lr_x, &mut lr_y
    ));
    println!(
        "ul_x: {}, ul_y: {}, lr_x: {}, lr_y: {}",
        ul_x, ul_y, lr_x, lr_y
    );
    Ok((ul_x as i32, ul_y as i32, lr_x as i32, lr_y as i32))
}

fn set_visible_area(handle: &FLIHandle, roi: &ROI) -> Result<(), Error> {
    let ul_x = roi.x_min as c_long;
    let ul_y = roi.y_min as c_long;
    let lr_x = (roi.x_min + roi.width) as c_long;
    let lr_y = (roi.y_min + roi.height) as c_long;
    FLICALL!(FLISetImageArea(*handle.0, ul_x, ul_y, lr_x, lr_y));
    Ok(())
}

fn get_readout_dim(handle: &FLIHandle) -> Result<ROI, Error> {
    let mut width: c_long = 0;
    let mut hoffset: c_long = 0;
    let mut hbin: c_long = 0;
    let mut height: c_long = 0;
    let mut voffset: c_long = 0;
    let mut vbin: c_long = 0;
    FLICALL!(FLIGetReadoutDimensions(
        *handle.0,
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

fn set_hbin(handle: &FLIHandle, bin: u32) -> Result<(), Error> {
    if !(0..16).contains(&bin) {
        return Err(Error::InvalidValue(format!(
            "Invalid horizontal binning value {}",
            bin
        )));
    }
    FLICALL!(FLISetHBin(*handle.0, bin as c_long));
    Ok(())
}

fn set_vbin(handle: &FLIHandle, bin: u32) -> Result<(), Error> {
    if !(0..16).contains(&bin) {
        return Err(Error::InvalidValue(format!(
            "Invalid vertical binning value {}",
            bin
        )));
    }
    FLICALL!(FLISetVBin(*handle.0, bin as c_long));
    Ok(())
}

fn get_serial(handle: &FLIHandle) -> Result<String, Error> {
    let mut serial = [0i8; 128];
    FLICALL!(FLIGetSerialString(
        *handle.0,
        serial.as_mut_ptr(),
        serial.len()
    ));
    let serial = unsafe { CStr::from_ptr(serial.as_ptr()) };
    Ok(serial.to_string_lossy().to_string())
}

fn get_camera_mode(handle: &FLIHandle) -> Result<(c_long, String), Error> {
    let mut mode = [0i8; 128];
    let mut modec: flimode_t = 0;
    FLICALL!(FLIGetCameraMode(*handle.0, &mut modec));
    FLICALL!(FLIGetCameraModeString(
        *handle.0,
        modec,
        mode.as_mut_ptr(),
        mode.len()
    ));
    let mode = unsafe { CStr::from_ptr(mode.as_ptr()) };
    Ok((modec, mode.to_string_lossy().to_string()))
}

fn get_pixel_size(handle: &FLIHandle) -> Result<(f64, f64), Error> {
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    FLICALL!(FLIGetPixelSize(*handle.0, &mut x, &mut y));
    Ok((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_temperature() {
        let cam = open_first_camera().unwrap();
        let temp = get_temperature(&cam.handle).unwrap();
        println!("Temperature: {}", temp);
        assert!(temp >= -100.0);
    }

    #[test]
    fn test_set_temperature() {
        let cam = open_first_camera().unwrap();
        set_temperature(&cam.handle, -20.0).unwrap();
    }

    #[test]
    fn test_get_cooler_power() {
        let cam = open_first_camera().unwrap();
        let power = get_cooler_power(&cam.handle).unwrap();
        println!("Cooler power: {}", power);
        assert!(power >= 0.0);
    }

    #[test]
    fn test_get_model() {
        let cam = open_first_camera().unwrap();
        let model = get_model(&cam.handle).unwrap();
        println!("Model: {}", model);
        assert!(!model.is_empty());
    }

    #[test]
    fn test_set_exposure() {
        let mut cam = open_first_camera().unwrap();
        set_exposure(&mut cam.handle, Duration::from_millis(100)).unwrap();
    }

    #[test]
    fn test_get_array_size() {
        let cam = open_first_camera().unwrap();
        let size = get_array_size(&cam.handle).unwrap();
        println!("Array size: {:?}", size);
        assert!(size.0 > 0 && size.1 > 0);
    }

    #[test]
    fn test_set_visible_area() {
        let mut cam = open_first_camera().unwrap();
        println!("{:?}", get_array_size(&cam.handle).unwrap());
        let roi = ROI {
            x_min: 100,
            y_min: 100,
            width: 100,
            height: 100,
            bin_x: 1,
            bin_y: 1,
        };
        set_visible_area(&cam.handle, &roi).unwrap();
        println!("{:?}", get_array_size(&cam.handle).unwrap());
        println!("{}", get_readout_dim(&cam.handle).unwrap());
        set_hbin(&cam.handle, 2).unwrap();
        set_vbin(&cam.handle, 2).unwrap();
        println!("{}", get_readout_dim(&cam.handle).unwrap());
    }

    #[test]
    fn test_get_serial() {
        let cam = open_first_camera().unwrap();
        let serial = get_serial(&cam.handle).unwrap();
        println!("Serial: {}", serial);
        assert!(!serial.is_empty());
    }

    #[test]
    fn test_get_camera_mode() {
        let cam = open_first_camera().unwrap();
        let mode = get_camera_mode(&cam.handle).unwrap();
        println!("Camera mode: {:?}", mode);
    }
}
