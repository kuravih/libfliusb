/*

  Copyright (c) 2002 Finger Lakes Instrumentation (FLI), L.L.C.
  All rights reserved.

  Redistribution and use in source and binary forms, with or without
  modification, are permitted provided that the following conditions
  are met:

        Redistributions of source code must retain the above copyright
        notice, this list of conditions and the following disclaimer.

        Redistributions in binary form must reproduce the above
        copyright notice, this list of conditions and the following
        disclaimer in the documentation and/or other materials
        provided with the distribution.

        Neither the name of Finger Lakes Instrumentation (FLI), LLC
        nor the names of its contributors may be used to endorse or
        promote products derived from this software without specific
        prior written permission.

  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
  ``AS IS'' AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
  LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS
  FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE
  REGENTS OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT,
  INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING,
  BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
  LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
  CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
  LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN
  ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
  POSSIBILITY OF SUCH DAMAGE.

  ======================================================================

  Finger Lakes Instrumentation, L.L.C. (FLI)
  web: http://www.fli-cam.com
  email: support@fli-cam.com

*/

#ifndef _LIBFLI_H_
#define _LIBFLI_H_

#include <sys/types.h>

/**
 * @brief Mark a variable as unused to suppress compiler warnings.
 * 
 */
#define FLI_UNUSED(x) ((void)(x))

/**
 * @brief An opaque handle used by library functions to refer to FLI
 * hardware.
 * 
 */
#define FLI_INVALID_DEVICE (-1)

/**
 * @brief An opaque handle used by library functions to refer to FLI hardware.
 * 
 */
typedef long flidev_t;

/**
 * @brief The domain of an FLI device.  This consists of a bitwise ORed
 * combination of interface method and device type.  Valid interfaces
 * are \texttt{FLIDOMAIN_PARALLEL_PORT}, \texttt{FLIDOMAIN_USB},
 * \texttt{FLIDOMAIN_SERIAL}, and \texttt{FLIDOMAIN_INET}.  Valid
 * device types are \texttt{FLIDEVICE_CAMERA},
 * \texttt{FLIDOMAIN_FILTERWHEEL}, and \texttt{FLIDOMAIN_FOCUSER}.
 * 
 * @see FLIOpen
 * @see FLIList
 * 
 */
typedef long flidomain_t;

/**
 * @brief Interface domain None
 * 
 */
#define FLIDOMAIN_NONE (0x00)
/**
 * @brief Parallel port interface domain
 * 
 */
#define FLIDOMAIN_PARALLEL_PORT (0x01)
/**
 * @brief USB interface domain
 * 
 */
#define FLIDOMAIN_USB (0x02)
/**
 * @brief Serial interface domain
 * 
 */
#define FLIDOMAIN_SERIAL (0x03)
/**
 * @brief Internet interface domain
 * 
 */
#define FLIDOMAIN_INET (0x04)
/**
 * @brief Serial interface domain 19200 baud
 * 
 */
#define FLIDOMAIN_SERIAL_19200 (0x05)
/**
 * @brief Serial interface domain 1200 baud
 * 
 */
#define FLIDOMAIN_SERIAL_1200 (0x06)
/**
 * @brief Serial interface domain mask (0x000f)
 * 
 */
#define FLIDOMAIN_INTERFACE_MASK (0x000f)


/**
 * @brief None-type device
 * 
 */
#define FLIDEVICE_NONE (0x000)
/**
 * @brief Camera device
 * 
 */
#define FLIDEVICE_CAMERA (0x100)
/**
 * @brief Filter wheel device
 * 
 */
#define FLIDEVICE_FILTERWHEEL (0x200)
/**
 * @brief Focuser device
 * 
 */
#define FLIDEVICE_FOCUSER (0x300)
/**
 * @brief High-speed filter wheel device
 * 
 */
#define FLIDEVICE_HS_FILTERWHEEL (0x0400)
/**
 * @brief Raw device
 * 
 */
#define FLIDEVICE_RAW (0x0f00)
/**
 * @brief Device mask (0x0f00)
 * 
 */
#define FLIDOMAIN_DEVICE_MASK (0x0f00)

/* The following two are really the same. ..._CONNECTION is old (deprecated) */
#define FLIDEVICE_ENUMERATE_BY_CONNECTION (0x8000)
#define FLIDEVICE_ENUMERATE_BY_SERIAL (0x8000)
#define FLIDOMAIN_OPTIONS_MASK (0xf000)


/**
 * @brief The frame type for an FLI CCD camera device.  Valid frame types are
 * #FLI_FRAME_TYPE_NORMAL and #FLI_FRAME_TYPE_DARK.
 * @link FLISetFrameType
 */
typedef long fliframe_t;

#define FLI_FRAME_TYPE_NORMAL (0)
#define FLI_FRAME_TYPE_DARK (1)
#define FLI_FRAME_TYPE_FLOOD (2)
#define FLI_FRAME_TYPE_RBI_FLUSH (FLI_FRAME_TYPE_FLOOD | FLI_FRAME_TYPE_DARK)

/**
   The gray-scale bit depth for an FLI camera device.  Valid bit
   depths are \texttt{FLI_MODE_8BIT} and \texttt{FLI_MODE_16BIT}.

   @see FLISetBitDepth
*/
typedef long flibitdepth_t;

#define FLI_MODE_8BIT (0)
#define FLI_MODE_16BIT (1)

/**
 * @brief Type used for shutter operations for an FLI camera device.  Valid
 * shutter types are FLI_SHUTTER_CLOSE,
 * FLI_SHUTTER_OPEN,
 * FLI_SHUTTER_EXTERNAL_TRIGGER,
 * FLI_SHUTTER_EXTERNAL_TRIGGER_LOW, and
 * FLI_SHUTTER_EXTERNAL_TRIGGER_HIGH.
 * 
 * @see FLIControlShutter
 * 
 */
typedef long flishutter_t;

#define FLI_SHUTTER_CLOSE (0x0000)
#define FLI_SHUTTER_OPEN (0x0001)
#define FLI_SHUTTER_EXTERNAL_TRIGGER (0x0002)
#define FLI_SHUTTER_EXTERNAL_TRIGGER_LOW (0x0002)
#define FLI_SHUTTER_EXTERNAL_TRIGGER_HIGH (0x0004)
#define FLI_SHUTTER_EXTERNAL_EXPOSURE_CONTROL (0x0008)

/**
   Type used for background flush operations for an FLI camera device.  Valid
   bgflush types are \texttt{FLI_BGFLUSH_STOP} and
   \texttt{FLI_BGFLUSH_START}.

   @see FLIControlBackgroundFlush
*/
typedef long flibgflush_t;

#define FLI_BGFLUSH_STOP (0x0000)
#define FLI_BGFLUSH_START (0x0001)

/**
   Type used to determine which temperature channel to read.  Valid
   channel types are \texttt{FLI_TEMPERATURE_INTERNAL} and
   \texttt{FLI_TEMPERATURE_EXTERNAL}.

   @see FLIReadTemperature
*/
typedef long flichannel_t;

#define FLI_TEMPERATURE_INTERNAL (0x0000)
#define FLI_TEMPERATURE_EXTERNAL (0x0001)
#define FLI_TEMPERATURE_CCD (0x0000)
#define FLI_TEMPERATURE_BASE (0x0001)

/**
   Type specifying library debug levels.  Valid debug levels are
   \texttt{FLIDEBUG_NONE}, \texttt{FLIDEBUG_INFO},
   \texttt{FLIDEBUG_WARN}, and \texttt{FLIDEBUG_FAIL}.

   @see FLISetDebugLevel
*/
typedef long flidebug_t;
typedef long flimode_t;
typedef long flistatus_t;
typedef long flitdirate_t;
typedef long flitdiflags_t;

/* Status settings */
#define FLI_CAMERA_STATUS_UNKNOWN (0xffffffff)
#define FLI_CAMERA_STATUS_MASK (0x00000003)
#define FLI_CAMERA_STATUS_IDLE (0x00)
#define FLI_CAMERA_STATUS_WAITING_FOR_TRIGGER (0x01)
#define FLI_CAMERA_STATUS_EXPOSING (0x02)
#define FLI_CAMERA_STATUS_READING_CCD (0x03)
#define FLI_CAMERA_DATA_READY (0x80000000)

#define FLI_FOCUSER_STATUS_UNKNOWN (0xffffffff)
#define FLI_FOCUSER_STATUS_HOMING (0x00000004)
#define FLI_FOCUSER_STATUS_MOVING_IN (0x00000001)
#define FLI_FOCUSER_STATUS_MOVING_OUT (0x00000002)
#define FLI_FOCUSER_STATUS_MOVING_MASK (0x00000007)
#define FLI_FOCUSER_STATUS_HOME (0x00000080)
#define FLI_FOCUSER_STATUS_LIMIT (0x00000040)
#define FLI_FOCUSER_STATUS_LEGACY (0x10000000)

#define FLI_FILTER_WHEEL_PHYSICAL (0x100)
#define FLI_FILTER_WHEEL_VIRTUAL (0)
#define FLI_FILTER_WHEEL_LEFT (FLI_FILTER_WHEEL_PHYSICAL | 0x00)
#define FLI_FILTER_WHEEL_RIGHT (FLI_FILTER_WHEEL_PHYSICAL | 0x01)
#define FLI_FILTER_STATUS_MOVING_CCW (0x01)
#define FLI_FILTER_STATUS_MOVING_CW (0x02)
#define FLI_FILTER_POSITION_UNKNOWN (0xff)
#define FLI_FILTER_POSITION_CURRENT (0x200)
#define FLI_FILTER_STATUS_HOMING (0x00000004)
#define FLI_FILTER_STATUS_HOME (0x00000080)
#define FLI_FILTER_STATUS_HOME_LEFT (0x00000080)
#define FLI_FILTER_STATUS_HOME_RIGHT (0x00000040)
#define FLI_FILTER_STATUS_HOME_SUCCEEDED (0x00000008)

#define FLIDEBUG_NONE (0x00)
#define FLIDEBUG_INFO (0x01)
#define FLIDEBUG_WARN (0x02)
#define FLIDEBUG_FAIL (0x04)
#define FLIDEBUG_IO		(0x08)
#define FLIDEBUG_ALL (FLIDEBUG_INFO | FLIDEBUG_WARN | FLIDEBUG_FAIL)

#define FLI_IO_P0 (0x01)
#define FLI_IO_P1 (0x02)
#define FLI_IO_P2 (0x04)
#define FLI_IO_P3 (0x08)

#define FLI_FAN_SPEED_OFF (0x00)
#define FLI_FAN_SPEED_ON (0xffffffff)

#define FLI_EEPROM_USER (0x00)
#define FLI_EEPROM_PIXEL_MAP (0x01)

#define FLI_PIXEL_DEFECT_COLUMN (0x00)
#define FLI_PIXEL_DEFECT_CLUSTER (0x10)
#define FLI_PIXEL_DEFECT_POINT_BRIGHT (0x20)
#define FLI_PIXEL_DEFECT_POINT_DARK (0x30)

#ifndef LIBFLIAPI
#  ifdef _WIN32
#    ifdef _LIB
#      define LIBFLIAPI long __stdcall
#    else
#      ifdef _USRDLL
/* The module definition file precludes using __declspec(dllexport) */
/*#        define LIBFLIAPI __declspec(dllexport) long __stdcall    */
#      define LIBFLIAPI long __stdcall
#      else
#        define LIBFLIAPI __declspec(dllimport) long __stdcall
#      endif
#    endif
#  else
#    define LIBFLIAPI long
#  endif
#endif

/* Library API Function prototypes */

#ifdef __cplusplus
extern "C" {  // only need to export C interface if used by C++ source code
#endif

#ifdef WIN32
void __cdecl FLIDebug(int level, char *format, ...);
#else
void FLIDebug(int level, char *format, ...);
#endif

/**
 * @brief Get a handle to an FLI device. This function requires the filename and domain of the requested device. Valid device filenames can be obtained using the `FLIList()` function. An application may use any number of handles associated with the same physical device. When doing so, it is important to lock the appropriate device to ensure that multiple accesses to the same device do not occur during critical operations.
 * 
 * @param dev Pointer to where a handle to the device will be stored.
 * @param name Pointer to a string where the device filename to be opened is stored.
 * For parallel port devices that are not probed by `FLIList()` (Win 9x), place the address of the parallel port in a string in ASCII form, e.g. "0x378".
 * @param domain Domain to apply to `name` for device opening. This is a bitwise OR of the interface method and device type. Valid interfaces are `FLIDOMAIN_PARALLEL_PORT`, `FLIDOMAIN_USB`, `FLIDOMAIN_SERIAL`, and `FLIDOMAIN_INET`. Valid device types are `FLIDEVICE_CAMERA`, `FLIDOMAIN_FILTERWHEEL`, and `FLIDOMAIN_FOCUSER`.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIOpen(flidev_t *dev, char *name, flidomain_t domain);

/**
 * @brief Enable debugging of API operations and communications. Use this function in 
 * combination with FLIDebug to assist in diagnosing problems that may be encountered 
 * during programming.
 * 
 * When usings Microsoft Windows operating systems, creating an empty file 
 * `C:\\FLIDBG.TXT` will override this option. All debug output will then 
 * be directed to this file.
 * 
 * @param host Name of the file to send debug output to. This parameter is ignored on Linux where `syslog(3)` is used to send debug output to the system log (see `syslog.conf(5)` to confugure `syslogd`).
 * @param level Debug level. This parameter is a bitwise OR of the following values:
 * - `FLIDEBUG_NONE` - No debug output.
 * - `FLIDEBUG_INFO` - Informational messages.
 * - `FLIDEBUG_WARN` - Warning messages.
 * - `FLIDEBUG_FAIL` - Error messages.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLISetDebugLevel(char *host, flidebug_t level);

/**
 * @brief Close a handle to an FLI device. This function releases the handle to the device and frees any resources associated with it.
 * 
 * @param dev Handle to the device to close.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIClose(flidev_t dev);

/**
 * @brief Get the current library version. This function copies up to `len - 1` characters of the library version string to the buffer pointed to by `ver`. The string is null-terminated.
 * 
 * @param ver Pointer to a character buffer where the library version string will be stored.
 * @param len The size of the buffer in bytes.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGetLibVersion(char* ver, size_t len);

/**
 * @brief Get the model of a given device. This function copies up to `len - 1` characters of the device model string to the buffer pointed to by `model`. The string is null-terminated.
 * 
 * @param dev Device handle.
 * @param model Pointer to a character buffer where the device model string will be stored.
 * @param len The size of the buffer in bytes.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGetModel(flidev_t dev, char* model, size_t len);

/**
 * @brief Find the dimensions of a pixel in the device's CCD array. This function returns the dimensions of a pixel in the device's CCD array in microns. The values are stored in the variables pointed to by `pixel_x` and `pixel_y`.
 * 
 * @param dev Camera handle.
 * @param pixel_x Pointer to a double where the pixel width will be stored.
 * @param pixel_y Pointer to a double where the pixel height will be stored.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGetPixelSize(flidev_t dev, double *pixel_x, double *pixel_y);

/**
 * @brief Get the hardware revision of a given device. This function returns the hardware revision of the device in the variable pointed to by `hwrev`.
 * 
 * @param dev Device handle.
 * @param hwrev Pointer to a long where the hardware revision will be stored.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGetHWRevision(flidev_t dev, long *hwrev);

/**
 * @brief Get the firmware revision of a given device. This function returns the firmware revision of the device in the variable pointed to by `fwrev`.
 * 
 * @param dev Device handle.
 * @param fwrev Pointer to a long where the firmware revision will be stored.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGetFWRevision(flidev_t dev, long *fwrev);

/**
 * @brief Get the array area of a given device. This function returns the array area of the device in the variables pointed to by `ul_x`, `ul_y`, `lr_x`, and `lr_y`.
 * 
 * @param dev Camera handle.
 * @param ul_x Upper-left x-coordinate.
 * @param ul_y Upper-left y-coordinate.
 * @param lr_x Lower-right x-coordinate.
 * @param lr_y Lower-right y-coordinate.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGetArrayArea(flidev_t dev, long* ul_x, long* ul_y,
			  long* lr_x, long* lr_y);

/**
 * @brief Get the visible area of a given device. This function returns the visible area of the device in the variables pointed to by `ul_x`, `ul_y`, `lr_x`, and `lr_y`.
 * 
 * @param dev Camera handle.
 * @param ul_x Upper-left x-coordinate.
 * @param ul_y Upper-left y-coordinate.
 * @param lr_x Lower-right x-coordinate.
 * @param lr_y Lower-right y-coordinate.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGetVisibleArea(flidev_t dev, long* ul_x, long* ul_y,
			    long* lr_x, long* lr_y);

/**
 * @brief Set the exposure time for a given device. This function sets the exposure time for the device to `exptime` milliseconds.
 * 
 * @param dev Camera handle.
 * @param exptime Exposure time in milliseconds.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLISetExposureTime(flidev_t dev, long exptime);

/**
 * @brief Set the region of interest for a given device. This function sets the region of interest for the device to the rectangle defined by the upper-left corner `(ul_x, ul_y)` and the lower-right corner `(lr_x, lr_y)`.
 * 
 * @param dev Camera handle.
 * @param ul_x Upper-left x-coordinate (unbinned coordinates).
 * @param ul_y Upper-left y-coordinate (unbinned coordinates).
 * @param lr_x Lower-right x-coordinate (binned coordinates).
 * @param lr_y Lower-right y-coordinate (binned coordinates).
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLISetImageArea(flidev_t dev, long ul_x, long ul_y,
			  long lr_x, long lr_y);

/**
 * @brief Set the horizontal binning for a given device. This function sets the horizontal binning for the device to `hbin`.
 * 
 * @param dev Camera handle.
 * @param hbin Horizontal bin value. Valid values are between 1 and 16.
 * @return LIBFLIAPI 
 */
LIBFLIAPI FLISetHBin(flidev_t dev, long hbin);

/**
 * @brief Set the vertical binning for a given device. This function sets the vertical binning for the device to `vbin`.
 * 
 * @param dev Camera handle.
 * @param vbin Vertical bin value. Valid values are between 1 and 16.
 * @return LIBFLIAPI 
 */
LIBFLIAPI FLISetVBin(flidev_t dev, long vbin);

/**
 * @brief Set the frame type for a given device. This function sets the frame type for the device to `frametype`.
 * 
 * @param dev Camera handle.
 * @param frametype Frame type. Valid values are `FLI_FRAME_TYPE_NORMAL` (shutter open) and `FLI_FRAME_TYPE_DARK` (shutter closed).
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLISetFrameType(flidev_t dev, fliframe_t frametype);

/**
 * @brief Cancel an ongoing exposure.
 * 
 * @param dev Camera handle.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLICancelExposure(flidev_t dev);

/**
 * @brief Get the remaining exposure time in milliseconds.
 * 
 * @param dev Camera handle.
 * @param timeleft Pointer to a long where the remaining exposure time will be stored.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGetExposureStatus(flidev_t dev, long *timeleft);

/**
 * @brief Set the temperature of a given device.
 * 
 * @param dev Camera handle.
 * @param temperature Temperture set point in degrees Celsius. Valid values are between -55 and 45.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLISetTemperature(flidev_t dev, double temperature);

/**
 * @brief Get the current temperature of a given device.
 * 
 * @param dev Camerea handle.
 * @param temperature Temperature of the camera cold finger in degrees Celsius.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGetTemperature(flidev_t dev, double *temperature);

/**
 * @brief Get the current power of the cooler in milliwatts.
 * 
 * @param dev Device handle.
 * @param power Power of the cooler in milliwatts.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGetCoolerPower(flidev_t dev, double *power);

/**
 * @brief Grab a row of an image. This function grabs the next available row of the image from the
 * camera device `dev`. The row of width `width` is stored in the buffer pointed to by `buff`.
 * The size of the buffer pointed to by `buff` must take into account the bit depth of the image,
 * meaning the buffer must be at least `width` bytes long for 8-bit images and `2 * width` bytes for
 * 16-bit images.
 * 
 * @param dev Camera handle.
 * @param buff Pointer to where the next row of the image will be stored.
 * @param width Row width in pixels.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGrabRow(flidev_t dev, void *buff, size_t width);

/**
 * @brief Expose a frame for a given camera. This function exposes a frame according to
 * the settings (image area, exposure time, bit depth etc.) of the camera `dev`. The settings
 * of `dev` must be valid for the camera device. They are set by calling the appropriate `set`
 * library functions.
 * Note: This function returns after the exposure has started.
 * 
 * @param dev Camera handle.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIExposeFrame(flidev_t dev);

/**
 * @brief Flush a number of rows for a given camera. This function flushes `rows` rows of the image
 * from the camera device `dev`. The rows are flushed `repeat` times.
 * 
 * @param dev Camera handle.
 * @param rows Number of rows to flush.
 * @param repeat Number of times to repeat the flush.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIFlushRow(flidev_t dev, long rows, long repeat);

/**
 * @brief Set the number of flushes for a given camera. This function sets the number of flushes
 * for the camera device `dev` to `nflushes`.
 * 
 * @param dev Camera handle.
 * @param nflushes Number of flushes.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLISetNFlushes(flidev_t dev, long nflushes);

/**
 * @brief Set the bit depth for a given camera. This function sets the bit depth for the camera
 * device `dev` to `bitdepth`.
 * 
 * @param dev Camera handle.
 * @param bitdepth Bit depth. Valid values are `FLI_MODE_8BIT` and `FLI_MODE_16BIT`.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLISetBitDepth(flidev_t dev, flibitdepth_t bitdepth);
LIBFLIAPI FLIReadIOPort(flidev_t dev, long *ioportset);
LIBFLIAPI FLIWriteIOPort(flidev_t dev, long ioportset);
LIBFLIAPI FLIConfigureIOPort(flidev_t dev, long ioportset);

/**
 * @brief Acquire an exclusive lock (`mutex`) on the device `dev`. This function
 * prevents other threads or processes from accessing the device while the lock is held.
 * 
 * @param dev Device handle.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLILockDevice(flidev_t dev);

/**
 * @brief Release the lock (`mutex`) on the device `dev`. This function allows other threads
 * or processes to access the device.
 * 
 * @param dev Device handle.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIUnlockDevice(flidev_t dev);

/**
 * @brief Control the shutter on a camera.
 * 
 * @param dev Camera handle.
 * @param shutter How to control the shutter. Valid values are 
 *    - `FLI_SHUTTER_CLOSE`: Close the shutter,
 *    - `FLI_SHUTTER_OPEN`: Open the shutter, 
 *    - `FLI_SHUTTER_EXTERNAL_TRIGGER`: Exposure starts on logic low on `I/O[0]`,
 *    - `FLI_SHUTTER_EXTERNAL_TRIGGER_LOW`: Same as `FLI_SHUTTER_EXTERNAL_TRIGGER`,
 *    - `FLI_SHUTTER_EXTERNAL_TRIGGER_HIGH`: Exposure starts on logic high on `I/O[0]`. May not be available on all cameras.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIControlShutter(flidev_t dev, flishutter_t shutter);

/**
 * @brief Enables background flushing of the CCD array. This function 
 * enables or disables background flushing of the CCD array. The background
 * flushing is stopped whenever `FLIExposeFrame()`/`FLIControlShutter()` is called.
 * Note: This function is only available on some cameras.
 * 
 * @param dev Camera handle.
 * @param bgflush Enable or disable background flushing. Valid values are
 *   - `FLI_BGFLUSH_STOP`: Disable background flushing,
 *   - `FLI_BGFLUSH_START`: Enable background flushing.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIControlBackgroundFlush(flidev_t dev, flibgflush_t bgflush);
LIBFLIAPI FLISetDAC(flidev_t dev, unsigned long dacset);

/**
 * @brief List all available devices in the given domain. 
 * This function returns a NULL-terminated list of all 
 * available devices in the given domain. The list is 
 * stored in the variable pointed to by `names`.
 * 
 * @param domain Domain to list devices for. Valid domains are
 *  - `FLIDOMAIN_PARALLEL_PORT`,
 *  - `FLIDOMAIN_USB`,
 *  - `FLIDOMAIN_SERIAL`, and
 *  - `FLIDOMAIN_INET`.
 * Valid device types are
 *  - `FLIDEVICE_CAMERA`,
 *  - `FLIDOMAIN_FILTERWHEEL`, and
 *  - `FLIDOMAIN_FOCUSER`.
 * @param names Pointer to where the list of device names will be stored.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIList(flidomain_t domain, char ***names);

/**
 * @brief Free a list of device names. This function frees the memory 
 * allocated for the list of device names.
 * 
 * @param names Pointer to the list of device names.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIFreeList(char **names);

LIBFLIAPI FLIGetFilterName(flidev_t dev, long filter, char *name, size_t len);
LIBFLIAPI FLISetActiveWheel(flidev_t dev, long wheel);
LIBFLIAPI FLIGetActiveWheel(flidev_t dev, long *wheel);

LIBFLIAPI FLISetFilterPos(flidev_t dev, long filter);
LIBFLIAPI FLIGetFilterPos(flidev_t dev, long *filter);
LIBFLIAPI FLIGetFilterCount(flidev_t dev, long *filter);

LIBFLIAPI FLIStepMotor(flidev_t dev, long steps);
LIBFLIAPI FLIStepMotorAsync(flidev_t dev, long steps);
LIBFLIAPI FLIGetStepperPosition(flidev_t dev, long *position);
LIBFLIAPI FLIGetStepsRemaining(flidev_t dev, long *steps);
LIBFLIAPI FLIHomeFocuser(flidev_t dev);

/**
 * @brief Create a list of devices in the given domain.
 * Use `FLIListFirst()` and `FLIListNext()` to iterate over the list.
 * Use `FLIDeleteList()` to free the list.
 * 
 * @param domain Domain to search for devices. Valid domains are
 * - `FLIDOMAIN_PARALLEL_PORT`,
 * - `FLIDOMAIN_USB`,
 * - `FLIDOMAIN_SERIAL`, and
 * - `FLIDOMAIN_INET`.
 * Supply `0` to search all domains.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLICreateList(flidomain_t domain);

/**
 * @brief Delete a list of devices.
 * 
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIDeleteList(void);

/**
 * @brief Get the first device in the list.
 * 
 * @param domain Pointer to the domain of the first device.
 * @param filename Pointer to a character buffer where the filename of the first device will be stored.
 * @param fnlen Size of the buffer pointed to by `filename`.
 * @param name Pointer to a character buffer where the name of the first device will be stored.
 * @param namelen Size of the buffer pointed to by `name`.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIListFirst(flidomain_t *domain, char *filename, size_t fnlen, char *name, size_t namelen);

/**
 * @brief Get the next device in the list.
 * 
 * @param domain Pointer to the domain of the next device.
 * @param filename Pointer to a character buffer where the filename of the next device will be stored.
 * @param fnlen Size of the buffer pointed to by `filename`.
 * @param name Pointer to a character buffer where the name of the next device will be stored.
 * @param namelen Size of the buffer pointed to by `name`.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIListNext(flidomain_t *domain, char *filename, size_t fnlen, char *name, size_t namelen);

/**
 * @brief Retrieve temperature from a focuser device.
 * 
 * @param dev Focuser handle.
 * @param channel Temperature source. Valid values are
 * - `FLI_TEMPERATURE_INTERNAL`: Internal temperature sensor,
 * - `FLI_TEMPERATURE_EXTERNAL`: External temperature sensor.
 * @param temperature Temperature in degrees Celsius.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIReadTemperature(flidev_t dev, flichannel_t channel, double *temperature);
LIBFLIAPI FLIGetFocuserExtent(flidev_t dev, long *extent);
LIBFLIAPI FLIUsbBulkIO(flidev_t dev, int ep, void *buf, long *len);
LIBFLIAPI FLIGetDeviceStatus(flidev_t dev, long *status);
LIBFLIAPI FLIGetCameraModeString(flidev_t dev, flimode_t mode_index, char *mode_string, size_t siz);
LIBFLIAPI FLIGetCameraMode(flidev_t dev, flimode_t *mode_index);
LIBFLIAPI FLISetCameraMode(flidev_t dev, flimode_t mode_index);
LIBFLIAPI FLIHomeDevice(flidev_t dev);

/**
 * @brief NOT IMPLEMENTED.
 * 
 * @param dev 
 * @param buff 
 * @param buffsize 
 * @param bytesgrabbed 
 * @return LIBFLIAPI 
 */
LIBFLIAPI FLIGrabFrame(flidev_t dev, void* buff, size_t buffsize, size_t* bytesgrabbed);
LIBFLIAPI FLISetTDI(flidev_t dev, flitdirate_t tdi_rate, flitdiflags_t flags);

/**
 * @brief Grab a video frame from a camera device. This function grabs a video frame from the camera device `dev` and stores it in the buffer pointed to by `buff`. The size of the buffer pointed to by `buff` must be at least `size` bytes long.
 * 
 * @param dev Camera handle.
 * @param buff Buffer where the video frame will be stored.
 * @param size Size of the buffer in bytes.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGrabVideoFrame(flidev_t dev, void *buff, size_t size);

/**
 * @brief Stop video mode for a camera device. This function stops video mode for the camera device `dev`.
 * 
 * @param dev Camera handle.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIStopVideoMode(flidev_t dev);

/**
 * @brief Start video mode for a camera device. This function starts video mode for the camera device `dev`.
 * 
 * @param dev Camera handle.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIStartVideoMode(flidev_t dev);

/**
 * @brief Get the serial number of a device. This function copies up to `len - 1` characters of the serial number of the device to the buffer pointed to by `serial`. The string is null-terminated.
 * 
 * @param dev Device handle.
 * @param serial Pointer to a character buffer where the serial number will be stored.
 * @param len Size of the buffer in bytes.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGetSerialString(flidev_t dev, char* serial, size_t len);

/**
 * @brief End an exposure for a given camera. This function causes the 
 * exposure to end and image download begins immediately.
 * 
 * @param dev Camera handle.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIEndExposure(flidev_t dev);

/**
 * @brief Trigger an exposure for a given camera. This function triggers 
 * an exposure for the camera device `dev` waiting for an external
 * trigger signal.
 * 
 * @param dev Camera handle.
 * @return LIBFLIAPI 
 */
LIBFLIAPI FLITriggerExposure(flidev_t dev);

/**
 * @brief Get the current fan speed of a given device.
 * 
 * @param dev Device handle.
 * @param fan_speed Fan speed in RPM.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLISetFanSpeed(flidev_t dev, long fan_speed);
LIBFLIAPI FLISetVerticalTableEntry(flidev_t dev, long index, long height, long bin, long mode);
LIBFLIAPI FLIGetVerticalTableEntry(flidev_t dev, long index, long *height, long *bin, long *mode);

/**
 * @brief Get the readout dimensions of a given device. This function returns the readout dimensions of the device in the variables pointed to by `width`, `hoffset`, `hbin`, `height`, `voffset`, and `vbin`.
 * 
 * @param dev Camera handle.
 * @param width Width of the readout area.
 * @param hoffset Horizontal offset of the readout area.
 * @param hbin Horizontal binning of the readout area.
 * @param height Height of the readout area.
 * @param voffset Vertical offset of the readout area.
 * @param vbin Vertical binning of the readout area.
 * @return LIBFLIAPI Zero on success, non-zero error code on failure.
 */
LIBFLIAPI FLIGetReadoutDimensions(flidev_t dev, long *width, long *hoffset, long *hbin, long *height, long *voffset, long *vbin);
LIBFLIAPI FLIEnableVerticalTable(flidev_t dev, long width, long offset, long flags);
LIBFLIAPI FLIReadUserEEPROM(flidev_t dev, long loc, long address, long length, void *rbuf);
LIBFLIAPI FLIWriteUserEEPROM(flidev_t dev, long loc, long address, long length, void *wbuf);

#ifdef __cplusplus
}
#endif

#endif /* _LIBFLI_H_ */
