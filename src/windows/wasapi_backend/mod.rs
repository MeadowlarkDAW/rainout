// Some code here is copied or adapted from CPAL's WASAPI implementation.
// In particular:
//   - https://github.com/RustAudio/cpal/blob/1cd0caf8b620acd52db7e1f02058266a555fadd6/src/host/wasapi/device.rs

mod com;

use core::ops::{Deref, DerefMut};
use lazy_static::lazy_static;
use winapi::shared::minwindef::DWORD;
use winapi::um::mmdeviceapi::IMMEndpoint;
use std::ffi::OsString;
use std::os::windows::prelude::OsStringExt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{io::Error as IoError, mem};
use std::{ptr, slice};
use winapi::shared::guiddef::GUID;
use winapi::shared::{ksmedia, mmreg, wtypes};
use winapi::um::audioclient::{IAudioClient, IID_IAudioClient};
use winapi::um::combaseapi::{CoTaskMemFree, PropVariantClear};
use winapi::{
    shared::devpkey,
    um::{
        combaseapi::{CoCreateInstance, CLSCTX_ALL},
        coml2api,
        mmdeviceapi::{
            eAll, CLSID_MMDeviceEnumerator, IMMDevice, IMMDeviceCollection, IMMDeviceEnumerator,
            DEVICE_STATE_ACTIVE,
        },
        winnt::HRESULT,
    },
    Interface,
};

use crate::{AudioBackend, AudioBackendInfo, AudioDeviceInfo, DefaultChannelLayout, DeviceID};

// TODO: CPAL has this as a crate-level constant. Maybe that'd be useful here?
struct SampleRate(u32);

const COMMON_SAMPLE_RATES: &'static [SampleRate] = &[
    SampleRate(5512),
    SampleRate(8000),
    SampleRate(11025),
    SampleRate(16000),
    SampleRate(22050),
    SampleRate(32000),
    SampleRate(44100),
    SampleRate(48000),
    SampleRate(64000),
    SampleRate(88200),
    SampleRate(96000),
    SampleRate(176400),
    SampleRate(192000),
];

fn check_result(result: HRESULT) -> Result<(), IoError> {
    if result < 0 {
        Err(IoError::from_raw_os_error(result))
    } else {
        Ok(())
    }
}

// Temporary code to throw away errors
fn check_result_empty(result: HRESULT) -> Result<(), ()> {
    match check_result(result) {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

lazy_static! {
    static ref ENUMERATOR: Enumerator = {
        // COM initialization is thread local, but we only need to have COM initialized in the
        // thread we create the objects in
        com::com_initialized();

        // building the devices enumerator object
        unsafe {
            let mut enumerator: *mut IMMDeviceEnumerator = ptr::null_mut();

            let hresult = CoCreateInstance(
                &CLSID_MMDeviceEnumerator,
                ptr::null_mut(),
                CLSCTX_ALL,
                &IMMDeviceEnumerator::uuidof(),
                &mut enumerator as *mut *mut IMMDeviceEnumerator as *mut _,
            );

            check_result(hresult).unwrap();
            Enumerator(enumerator)
        }
    };
}

// TODO: Equivalent rusty-daw-io representation
enum SampleFormat {
    I16,
    F32,
}

struct Format {
    channels: u16,
    sample_rate: SampleRate,
    sample_format: SampleFormat,
}

enum WaveFormat {
    Ex(mmreg::WAVEFORMATEX),
    Extensible(mmreg::WAVEFORMATEXTENSIBLE),
}

// Use RAII to make sure CoTaskMemFree is called when we are responsible for freeing.
struct WaveFormatExPtr(*mut mmreg::WAVEFORMATEX);

impl Drop for WaveFormatExPtr {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(self.0 as *mut _);
        }
    }
}

impl WaveFormat {
    // Given a pointer to some format, returns a valid copy of the format.
    pub fn copy_from_waveformatex_ptr(ptr: *const mmreg::WAVEFORMATEX) -> Option<Self> {
        unsafe {
            match (*ptr).wFormatTag {
                mmreg::WAVE_FORMAT_PCM | mmreg::WAVE_FORMAT_IEEE_FLOAT => {
                    Some(WaveFormat::Ex(*ptr))
                }
                mmreg::WAVE_FORMAT_EXTENSIBLE => {
                    let extensible_ptr = ptr as *const mmreg::WAVEFORMATEXTENSIBLE;
                    Some(WaveFormat::Extensible(*extensible_ptr))
                }
                _ => None,
            }
        }
    }

    // Get the pointer to the WAVEFORMATEX struct.
    pub fn as_ptr(&self) -> *const mmreg::WAVEFORMATEX {
        self.deref() as *const _
    }
}

impl Deref for WaveFormat {
    type Target = mmreg::WAVEFORMATEX;
    fn deref(&self) -> &Self::Target {
        match *self {
            WaveFormat::Ex(ref f) => f,
            WaveFormat::Extensible(ref f) => &f.Format,
        }
    }
}

impl DerefMut for WaveFormat {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match *self {
            WaveFormat::Ex(ref mut f) => f,
            WaveFormat::Extensible(ref mut f) => &mut f.Format,
        }
    }
}

// Get a WaveFormat from a WAVEFORMATEX.
unsafe fn format_from_waveformatex_ptr(
    waveformatex_ptr: *const mmreg::WAVEFORMATEX,
) -> Option<Format> {
    fn cmp_guid(a: &GUID, b: &GUID) -> bool {
        a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
    }
    let sample_format = match ((*waveformatex_ptr).wBitsPerSample, (*waveformatex_ptr).wFormatTag) {
        (16, mmreg::WAVE_FORMAT_PCM) => SampleFormat::I16,
        (32, mmreg::WAVE_FORMAT_IEEE_FLOAT) => SampleFormat::F32,
        (n_bits, mmreg::WAVE_FORMAT_EXTENSIBLE) => {
            let waveformatextensible_ptr = waveformatex_ptr as *const mmreg::WAVEFORMATEXTENSIBLE;
            let sub = (*waveformatextensible_ptr).SubFormat;
            if n_bits == 16 && cmp_guid(&sub, &ksmedia::KSDATAFORMAT_SUBTYPE_PCM) {
                SampleFormat::I16
            } else if n_bits == 32 && cmp_guid(&sub, &ksmedia::KSDATAFORMAT_SUBTYPE_IEEE_FLOAT) {
                SampleFormat::F32
            } else {
                return None;
            }
        }
        // Unknown data format returned by GetMixFormat.
        _ => return None,
    };

    let format = Format {
        channels: (*waveformatex_ptr).nChannels as _,
        sample_rate: SampleRate((*waveformatex_ptr).nSamplesPerSec),
        sample_format,
    };
    Some(format)
}

/// RAII objects around `IMMDeviceEnumerator`.
struct Enumerator(*mut IMMDeviceEnumerator);

unsafe impl Send for Enumerator {}
unsafe impl Sync for Enumerator {}

/// Wrapper because of that stupid decision to remove `Send` and `Sync` from raw pointers.
#[derive(Copy, Clone)]
struct IAudioClientWrapper(*mut IAudioClient);
unsafe impl Send for IAudioClientWrapper {}
unsafe impl Sync for IAudioClientWrapper {}

struct Endpoint {
    endpoint: *mut IMMEndpoint,
}

unsafe fn immendpoint_from_immdevice(device: *const IMMDevice) -> *mut IMMEndpoint {
    let mut endpoint: *mut IMMEndpoint = ptr::null_mut();
    check_result(
        (*device).QueryInterface(&IMMEndpoint::uuidof(), &mut endpoint as *mut _ as *mut _),
    )
    .expect("could not query IMMDevice interface for IMMEndpoint");
    endpoint
}

/// An opaque type that identifies an end point.
pub struct Device {
    device: *mut IMMDevice,
    /// We cache an uninitialized `IAudioClient` so that we can call functions from it without
    /// having to create/destroy audio clients all the time.
    future_audio_client: Arc<Mutex<Option<IAudioClientWrapper>>>, // TODO: add NonZero around the ptr
}

impl Device {
    #[inline]
    fn from_imm_device(device: *mut IMMDevice) -> Self {
        Device { device, future_audio_client: Arc::new(Mutex::new(None)) }
    }

    /// Ensures that `future_audio_client` contains a `Some` and returns a locked mutex to it.
    fn ensure_future_audio_client(
        &self,
    ) -> Result<MutexGuard<Option<IAudioClientWrapper>>, IoError> {
        let mut lock = self.future_audio_client.lock().unwrap();
        if lock.is_some() {
            return Ok(lock);
        }

        let audio_client: *mut IAudioClient = unsafe {
            let mut audio_client = ptr::null_mut();
            let hresult = (*self.device).Activate(
                &IID_IAudioClient,
                CLSCTX_ALL,
                ptr::null_mut(),
                &mut audio_client,
            );

            // can fail if the device has been disconnected since we enumerated it, or if
            // the device doesn't support playback for some reason
            check_result(hresult)?;
            assert!(!audio_client.is_null());
            audio_client as *mut _
        };

        *lock = Some(IAudioClientWrapper(audio_client));
        Ok(lock)
    }

    pub fn device_name(&self) -> Result<String, ()> {
        unsafe {
            let mut device = self.device;

            // Open the device's property store
            let mut property_store = ptr::null_mut();
            (*device).OpenPropertyStore(coml2api::STGM_READ, &mut property_store);

            // Get the endpoint's friendly-name property.
            let mut property_value = mem::zeroed();
            if let Err(err) = check_result_empty((*property_store).GetValue(
                &devpkey::DEVPKEY_Device_FriendlyName as *const _ as *const _,
                &mut property_value,
            )) {
                // let description = format!("failed to retrieve name from property store: {}", err);
                // let err = BackendSpecificError { description };
                // return Err(err.into());
                return Err(());
            }

            // Read the friendly-name from the union data field, expecting a *const u16.
            if property_value.vt != wtypes::VT_LPWSTR as _ {
                // let description = format!(
                //     "property store produced invalid data: {:?}",
                //     property_value.vt
                // );
                // let err = BackendSpecificError { description };
                // return Err(err.into());
                return Err(());
            }
            let ptr_utf16 = *(&property_value.data as *const _ as *const *const u16);

            // Find the length of the friendly name.
            let mut len = 0;
            while *ptr_utf16.offset(len) != 0 {
                len += 1;
            }

            // Create the utf16 slice and covert it into a string.
            let name_slice = slice::from_raw_parts(ptr_utf16, len as usize);
            let name_os_string: OsString = OsStringExt::from_wide(name_slice);
            let name_string = match name_os_string.into_string() {
                Ok(string) => string,
                Err(os_string) => os_string.to_string_lossy().into(),
            };

            // Clean up the property.
            PropVariantClear(&mut property_value);

            Ok(name_string)
        }
    }

    // There is no way to query the list of all formats that are supported by the
    // audio processor, so instead we just trial some commonly supported formats.
    //
    // Common formats are trialed by first getting the default format (returned via
    // `GetMixFormat`) and then mutating that format with common sample rates and
    // querying them via `IsFormatSupported`.
    //
    // When calling `IsFormatSupported` with the shared-mode audio engine, only the default
    // number of channels seems to be supported. Any, more or less returns an invalid
    // parameter error. Thus, we just assume that the default number of channels is the only
    // number supported.
    fn supported_formats(&self) -> Result<SupportedInputConfigs, ()> {
        // SupportedStreamConfigsError
        // initializing COM because we call `CoTaskMemFree` to release the format.
        com::com_initialized();

        // Retrieve the `IAudioClient`.
        let lock = match self.ensure_future_audio_client() {
            Ok(lock) => lock,
            // Err(ref e) if e.raw_os_error() == Some(AUDCLNT_E_DEVICE_INVALIDATED) => {
            //     return Err(SupportedStreamConfigsError::DeviceNotAvailable)
            // }
            // Err(e) => {
            //     let description = format!("{}", e);
            //     let err = BackendSpecificError { description };
            //     return Err(err.into());
            // }
            Err(_) => return Err(()),
        };
        let client = lock.unwrap().0;

        unsafe {
            // Retrieve the pointer to the default WAVEFORMATEX.
            let mut default_waveformatex_ptr = WaveFormatExPtr(ptr::null_mut());
            match check_result((*client).GetMixFormat(&mut default_waveformatex_ptr.0)) {
                Ok(()) => (),
                // Err(ref e) if e.raw_os_error() == Some(AUDCLNT_E_DEVICE_INVALIDATED) => {
                //     return Err(SupportedStreamConfigsError::DeviceNotAvailable);
                // }
                // Err(e) => {
                //     let description = format!("{}", e);
                //     let err = BackendSpecificError { description };
                //     return Err(err.into());
                // }
                Err(_) => return Err(()),
            };

            // If the default format can't succeed we have no hope of finding other formats.
            assert_eq!(is_format_supported(client, default_waveformatex_ptr.0)?, true);

            // Copy the format to use as a test format (as to avoid mutating the original format).
            let mut test_format = {
                match Format::copy_from_waveformatex_ptr(default_waveformatex_ptr.0) {
                    Some(f) => f,
                    // If the format is neither EX nor EXTENSIBLE we don't know how to work with it.
                    None => return Ok(vec![].into_iter()),
                }
            };

            // Begin testing common sample rates.
            //
            // NOTE: We should really be testing for whole ranges here, but it is infeasible to
            // test every sample rate up to the overflow limit as the `IsFormatSupported` method is
            // quite slow.
            let mut supported_sample_rates: Vec<u32> = Vec::new();
            for &rate in COMMON_SAMPLE_RATES {
                let rate = rate.0 as DWORD;
                test_format.nSamplesPerSec = rate;
                test_format.nAvgBytesPerSec =
                    rate * u32::from((*default_waveformatex_ptr.0).nBlockAlign);
                if is_format_supported(client, test_format.as_ptr())? {
                    supported_sample_rates.push(rate);
                }
            }

            // If the common rates don't include the default one, add the default.
            let default_sr = (*default_waveformatex_ptr.0).nSamplesPerSec as _;
            if !supported_sample_rates.iter().any(|&r| r == default_sr) {
                supported_sample_rates.push(default_sr);
            }

            // Reset the sample rate on the test format now that we're done.
            test_format.nSamplesPerSec = (*default_waveformatex_ptr.0).nSamplesPerSec;
            test_format.nAvgBytesPerSec = (*default_waveformatex_ptr.0).nAvgBytesPerSec;

            // TODO: Test the different sample formats?

            // Create the supported formats.
            let format = match format_from_waveformatex_ptr(default_waveformatex_ptr.0) {
                Some(fmt) => fmt,
                None => {
                    let description =
                        "could not create a `cpal::SupportedStreamConfig` from a `WAVEFORMATEX`"
                            .to_string();
                    let err = BackendSpecificError { description };
                    return Err(err.into());
                }
            };
            let mut supported_formats = Vec::with_capacity(supported_sample_rates.len());
            for rate in supported_sample_rates {
                supported_formats.push(SupportedStreamConfigRange {
                    channels: format.channels.clone(),
                    min_sample_rate: SampleRate(rate as _),
                    max_sample_rate: SampleRate(rate as _),
                    buffer_size: format.buffer_size.clone(),
                    sample_format: format.sample_format.clone(),
                })
            }
            Ok(supported_formats.into_iter())
        }
    }

    pub fn supported_input_configs(
        &self,
    ) -> Result<SupportedInputConfigs, SupportedStreamConfigsError> {
        if self.data_flow() == eCapture {
            self.supported_formats()
        // If it's an output device, assume no input formats.
        } else {
            Ok(vec![].into_iter())
        }
    }

    pub fn supported_output_configs(
        &self,
    ) -> Result<SupportedOutputConfigs, SupportedStreamConfigsError> {
        if self.data_flow() == eRender {
            self.supported_formats()
        // If it's an input device, assume no output formats.
        } else {
            Ok(vec![].into_iter())
        }
    }
}

// ---

// TODO: Error handling?
pub fn backend_info() -> Result<AudioBackendInfo, ()> {
    let mut devices = Vec::new();

    unsafe {
        let mut collection: *mut IMMDeviceCollection = ptr::null_mut();

        (*ENUMERATOR.0).EnumAudioEndpoints(eAll, DEVICE_STATE_ACTIVE, &mut collection);

        let count = 0u32;
        // can fail if the parameter is null, which should never happen
        check_result_empty((*collection).GetCount(&count))?;

        for i in 0..count {
            // Get the device
            let mut imm_device: *mut IMMDevice = ptr::null_mut();
            check_result_empty((*collection).Item(i, &mut imm_device))?;

            let mut device = Device::from_imm_device(imm_device);

            let name_string = device.device_name()?;

            let info = AudioDeviceInfo {
                id: DeviceID { name: name_string, unique_id: None },
                in_ports: Vec::new(),      // TODO
                out_ports: Vec::new(),     // TODO
                sample_rates: vec![44100], // TODO: WASAPI won't tell us directly what sample rates are supported, so we'll need to guess and check - see CPAL source
                default_sample_rate: 44100,
                fixed_buffer_size_range: None, // TODO: I think this is correct? It's a little more complicated though. WASAPI takes a buffer size parameter, but it seems like a lower bound? I don't understand enough to know what I'm reading, but see https://stackoverflow.com/questions/20371033/wasapi-capture-buffer-size
                default_input_layout: DefaultChannelLayout::Unspecified, // TODO
                default_output_layout: DefaultChannelLayout::Unspecified, // TODO
            };

            devices.push(info);
        }
    }

    Ok(AudioBackendInfo {
        backend: AudioBackend::Wasapi,
        version: None,
        running: true,
        devices,
        default_device: None, // TODO
    })
}
