#![cfg(not(windows))]
mod fli_ffi;
#[macro_use]
mod flihandle;

mod flicamera;

// pub use flicamera::{
//     get_camera_ids, num_cameras, open_camera, open_first_camera, ASICameraProps, ASIImageFormat,
//     CameraInfoASI, CameraUnitASI,
// };

/// Re-export of [`cameraunit`] crate.
pub use cameraunit::{
    CameraInfo, CameraUnit, DynamicSerialImage, Error, ImageMetaData, OptimumExposure,
    OptimumExposureBuilder, SerialImageBuffer, ROI,
};

pub use flicamera::{
    get_camera_ids, num_cameras, open_camera, open_first_camera, CameraInfoFLI, CameraUnitFLI,
};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_camera_ids() {
        let ids = get_camera_ids();
        println!("Camera IDs: {:?}", ids);
        assert!(ids.is_ok());
    }

    #[test]
    fn test_num_cameras() {
        let num = num_cameras();
        println!("Number of cameras: {}", num);
        assert!(num >= 0);
    }
}
