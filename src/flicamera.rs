#![warn(missing_docs)]

use std::{
    ffi::{c_void, CStr, CString},
    str,
    sync::{atomic::Ordering, Arc},
    thread::sleep,
    time::{Duration, SystemTime},
};

use crate::fli_ffi::*;

use cameraunit::{CameraInfo, CameraUnit, DynamicSerialImage, Error, ImageMetaData, PixelBpp, ROI};
use log::warn;

use crate::flihandle::*;
/// This object describes a FLI camera, and provides methods for control and image capture.
///
/// This object implements the [`cameraunit::CameraUnit`] and [`cameraunit::CameraInfo`] trait.
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
/// This object describes a FLI camera and provides methods for obtaining housekeeping data.
///
/// This object implements the [`cameraunit::CameraInfo`] trait, and additionally the [`std::clone::Clone`] trait.
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

/// Get the IDs and names of the available ZWO ASI cameras.
///
/// # Examples
///
/// ```
/// let cam_ids = cameraunit_fli::get_camera_ids();
/// if let Ok(cam_ids) = cam_ids {
///     // do stuff with camera IDs and names
/// }
/// ```
pub fn get_camera_ids() -> Result<Vec<String>, Error> {
    let mut ptr = std::ptr::null_mut();
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

/// Get the number of available ZWO ASI cameras.
///
/// # Examples
///
/// ```
/// let num_cameras = cameraunit_fli::num_cameras();
/// if num_cameras <= 0 {
///     println!("No cameras found");
/// }
/// // proceed to get camera IDs and information
/// ```
pub fn num_cameras() -> i32 {
    let camlist = get_camera_ids();
    match camlist {
        Ok(list) => list.len() as i32,
        Err(_) => 0,
    }
}

/// Open a ZWO ASI camera by ID for access.
///
/// This method, if successful, returns a tuple containing a `CameraUnit_FLI` object and a `CameraInfo_FLI` object.
/// The `CameraUnit_FLI` object allows for control of the camera and image capture, while the `CameraInfo_FLI` object
/// only allows for access to housekeeping data.
///
/// The `CameraUnit_FLI` object is required for image capture, and should
/// be mutable in order to set exposure, ROI, gain, etc.
///
/// The `CameraInfo_FLI` object allows cloning and sharing, and is useful for obtaining housekeeping data from separate
/// threads.
///
/// # Arguments
///
/// * `id` - The ID of the camera to open. This ID can be obtained from the `get_camera_ids()` method.
///
/// # Errors
///  - [`cameraunit::Error::InvalidFormat`] - The ID provided is not valid.
///  - [`cameraunit::Error::GeneralError`] - Could not open camera (error code).
///  - [`cameraunit::Error::NoCamerasAvailable`] - No cameras are available.
///
/// # Examples
///
/// ```
/// use cameraunit_fli::open_camera;
/// let id = "FLI-04"; // some ID obtained using get_camera_ids()
/// if let Ok((mut cam, caminfo)) = open_camera(id) {
///
/// }
/// // do things with cam
/// ```
pub fn open_camera(name: &str) -> Result<(CameraUnitFLI, CameraInfoFLI), Error> {
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
    if handle.set_bpp(PixelBpp::Bpp16).is_err() {
        warn!("Error setting pixel bit depth to 16 bits, attempting 8 bits");
    } else if handle.set_bpp(PixelBpp::Bpp8).is_err() {
        warn!("Error setting pixel bit depth to 8 bits");
        handle.bpp.store(PixelBpp::Bpp16 as u32, Ordering::SeqCst);
    }
    let (x_min, y_min, x_max, y_max) = handle.get_array_size()?;

    let info = CameraInfoFLI {
        handle: handle.clone(),
        width: (x_max - x_min) as u32,
        height: (y_max - y_min) as u32,
        name: name.to_string(),
        serial,
    };

    Ok((
        CameraUnitFLI {
            handle,
            info: info.clone(),
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
        },
        info,
    ))
}

/// Open the first available ZWO ASI camera for access.
///
/// This method, if successful, returns a tuple containing a `CameraUnit_FLI` object and a `CameraInfo_FLI` object.
/// The `CameraUnit_FLI` object allows for control of the camera and image capture, while the `CameraInfo_FLI` object
/// only allows for access to housekeeping data.
///
/// The `CameraUnit_FLI` object is required for image capture, and should
/// be mutable in order to set exposure, ROI, gain, etc.
///
/// The `CameraInfo_FLI` object allows cloning and sharing, and is useful for obtaining housekeeping data from separate
/// threads.
///
/// # Errors
///  - [`cameraunit::Error::InvalidFormat`] - The ID provided is not valid.
///  - [`cameraunit::Error::GeneralError`] - Could not open camera (error code).
///  - [`cameraunit::Error::NoCamerasAvailable`] - No cameras are available.
///
/// # Examples
///
/// ```
/// use cameraunit_fli::open_first_camera;
///
/// if let Ok((mut cam, caminfo)) = open_first_camera() {
///
/// }
/// ```
pub fn open_first_camera() -> Result<(CameraUnitFLI, CameraInfoFLI), Error> {
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
        self.handle.get_temperature().ok()
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
        self.info.width
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
        Some(&self.handle.dev)
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
            FLICALL!(FLISetFrameType(
                self.handle.dev,
                if open {
                    FLI_FRAME_TYPE_NORMAL as i64
                } else {
                    FLI_FRAME_TYPE_DARK as i64
                }
            ));
            self.handle.dark.store(open, Ordering::SeqCst);
            Ok(open)
        }
    }

    fn get_shutter_open(&self) -> Result<bool, Error> {
        Ok(self.handle.dark.load(Ordering::SeqCst))
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
        self.start_exposure()?;
        sleep(self.get_exposure());
        while !self.image_ready()? {
            sleep(Duration::from_millis(10));
        }
        self.download_image()
    }

    fn start_exposure(&self) -> Result<(), Error> {
        if !self.handle.capturing.load(Ordering::SeqCst) {
            self.handle.capturing.store(true, Ordering::SeqCst);
            FLICALL!(FLIExposeFrame(self.handle.dev));
            self.handle.ready.store(false, Ordering::SeqCst);
            sleep(Duration::from_millis(100));
            Ok(())
        } else {
            Err(Error::ExposureInProgress)
        }
    }

    fn download_image(&self) -> Result<DynamicSerialImage, Error> {
        let rdy = self.handle.image_ready()?;
        if rdy {
            let width = self.roi.width;
            let height = self.roi.height;
            let mut buf = vec![0u16; (width * height) as usize];
            let mut grabbed = 0;
            let res = unsafe {
                FLIGrabFrame(
                    self.handle.dev,
                    buf.as_mut_ptr() as *mut c_void,
                    (width * height * 2) as usize,
                    &mut grabbed,
                )
            };
            if res != 0 {
                self.handle.capturing.store(false, Ordering::SeqCst);
                self.handle.ready.store(false, Ordering::SeqCst);
                return Err(Error::GeneralError(format!(
                    "Error grabbing frame: {}",
                    res
                )));
            }
            self.handle.capturing.store(false, Ordering::SeqCst);
            self.handle.ready.store(true, Ordering::SeqCst);
            let mut meta = ImageMetaData::default();
            meta.timestamp = SystemTime::now();
            meta.exposure = self.get_exposure();
            meta.temperature = self.handle.get_temperature()?;
            meta.camera_name = self.info.camera_name().to_string();
            meta.bin_x = self.roi.bin_x;
            meta.bin_y = self.roi.bin_y;
            meta.img_left = self.roi.x_min;
            meta.img_top = self.roi.y_min;
            let mut img = DynamicSerialImage::from_vec_u16(width as usize, height as usize, buf)
                .map_err(|e| Error::GeneralError(format!("Error creating image: {}", e)))?;
            img.set_metadata(meta);
            Ok(img)
        } else {
            Err(Error::ExposureInProgress)
        }
    }

    fn image_ready(&self) -> Result<bool, Error> {
        self.handle.image_ready()
    }

    fn set_exposure(&mut self, exposure: Duration) -> Result<Duration, Error> {
        if self.handle.capturing.load(Ordering::SeqCst) {
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
        Duration::from_millis(self.handle.exp.load(Ordering::SeqCst))
    }

    fn set_roi(&mut self, roi: &ROI) -> Result<&ROI, Error> {
        if self.info.is_capturing() {
            Err(Error::ExposureInProgress)
        } else {
            if roi.width == 0 && roi.height == 0 && roi.x_min == 0 && roi.y_min == 0 {
                return self.roi_reset();
            }
            if (roi.width == 0 || roi.height == 0) && (roi.x_min != 0 || roi.y_min != 0) {
                return Err(Error::InvalidValue("Invalid ROI".to_string()));
            }

            if roi.width * roi.bin_x > self.info.width || roi.height * roi.bin_y > self.info.height
            {
                return Err(Error::InvalidValue("Invalid ROI".to_string()));
            }

            let x_min = (roi.x_min * self.roi.bin_x) as i64;
            let y_min = (roi.y_min * self.roi.bin_y) as i64;

            let ul_x = self.x_min as i64 + x_min;
            let ul_y = self.y_min as i64 + y_min;

            if ul_x < self.x_min.into()
                || ul_x >= self.x_max.into()
                || ul_y < self.y_min.into()
                || ul_y >= self.y_max.into()
            {
                return Err(Error::InvalidValue("Invalid ROI".to_string()));
            }

            let lr_x = ul_x + roi.width as i64;
            let lr_y = ul_y + roi.height as i64;

            FLICALL!(FLISetImageArea(self.handle.dev, ul_x, ul_y, lr_x, lr_y));
            self.handle.set_hbin(roi.bin_x)?;
            self.handle.set_vbin(roi.bin_y)?;

            self.roi = self.handle.get_readout_dim()?;
            self.roi.x_min = (self.roi.x_min - self.x_min as u32) / self.roi.bin_x;
            self.roi.y_min = (self.roi.y_min - self.y_min as u32) / self.roi.bin_y;
            Ok(&self.roi)
        }
    }

    fn get_roi(&self) -> &ROI {
        &self.roi
    }

    fn set_bpp(&mut self, bpp: PixelBpp) -> Result<PixelBpp, Error> {
        self.handle.set_bpp(bpp)?;
        Ok(bpp)
    }

    fn get_bpp(&self) -> cameraunit::PixelBpp {
        self.handle.get_bpp()
    }
}

impl CameraUnitFLI {
    fn roi_reset(&mut self) -> Result<&ROI, Error> {
        self.handle.set_hbin(1)?;
        self.handle.set_vbin(1)?;
        FLICALL!(FLISetImageArea(
            self.handle.dev,
            self.x_min.into(),
            self.y_min.into(),
            self.x_max.into(),
            self.y_max.into()
        ));
        self.roi = self.handle.get_readout_dim()?;
        self.roi.x_min = 0;
        self.roi.y_min = 0;

        Ok(&self.roi)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::c_long;

    use super::*;

    #[test]
    fn test_get_temperature() {
        if let Ok((cam, _)) = open_first_camera() {
            let temp = cam.handle.get_temperature().unwrap();
            println!("Temperature: {}", temp);
            assert!(temp >= -100.);
        }
    }

    #[test]
    fn test_set_temperature() {
        if let Ok((cam, _)) = open_first_camera() {
            cam.handle.set_temperature(-20.).unwrap();
        }
    }

    #[test]
    fn test_get_cooler_power() {
        if let Ok((cam, _)) = open_first_camera() {
            let power = cam.handle.get_cooler_power().unwrap();
            println!("Cooler power: {}", power);
            assert!(power >= 0.);
        }
    }

    #[test]
    fn test_get_model() {
        if let Ok((cam, _)) = open_first_camera() {
            let model = cam.handle.get_model().unwrap();
            println!("Model: {}", model);
            assert!(!model.is_empty());
        }
    }

    #[test]
    fn test_set_exposure() {
        if let Ok((cam, _)) = open_first_camera() {
            cam.handle.set_exposure(Duration::from_millis(100)).unwrap();
        }
    }

    #[test]
    fn test_get_array_size() {
        if let Ok((cam, _)) = open_first_camera() {
            let size = cam.handle.get_array_size().unwrap();
            println!("Array size: {:?}", size);
            assert!(size.0 > 0 && size.1 > 0);
        }
    }

    #[test]
    fn test_set_visible_area() {
        if let Ok((cam, _)) = open_first_camera() {
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
    }

    #[test]
    fn test_get_serial() {
        if let Ok((cam, _)) = open_first_camera() {
            let serial = cam.handle.get_serial().unwrap();
            println!("Serial: {}", serial);
            assert!(!serial.is_empty());
        }
    }

    #[test]
    fn test_get_camera_mode() {
        if let Ok((cam, _)) = open_first_camera() {
            let mode = cam.handle.list_camera_modes();
            println!("Camera modes: {:?}", mode);
        }
    }

    #[test]
    fn test_get_exposure_status() {
        if let Ok((mut cam, _)) = open_first_camera() {
            println!("{}", cam.get_roi());
            let mut timeleft: c_long = 0;
            let res = unsafe { FLIGetExposureStatus(cam.handle.dev, &mut timeleft) };
            println!("Exposure status: {}", res);
            println!("Time left: {}", timeleft);
            let res = unsafe { FLIGetDeviceStatus(cam.handle.dev, &mut timeleft) };
            println!("Device status ({}): {}", res, timeleft);
            cam.set_exposure(Duration::from_millis(100)).unwrap();
            cam.set_roi(&ROI {
                x_min: 100,
                y_min: 100,
                width: 300,
                height: 500,
                bin_x: 2,
                bin_y: 2,
            })
            .unwrap();
            cam.start_exposure().unwrap();
            let res = unsafe { FLIGetExposureStatus(cam.handle.dev, &mut timeleft) };
            println!("Exposure status: {}, time left: {}", res, timeleft);
            let mut count = 100;
            while !cam.image_ready().unwrap() && count > 0 {
                println!("Waiting for image... {}", count);
                count -= 1;
                sleep(Duration::from_millis(100));
            }
            println!("Image ready");
            let img = cam.download_image().unwrap();
            img.save("test.png").unwrap();
        }
    }
}
