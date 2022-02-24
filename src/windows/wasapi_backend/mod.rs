// Some code here is copied or adapted from CPAL's WASAPI implementation.

mod com;

use lazy_static::lazy_static;
use std::ffi::OsString;
use std::os::windows::prelude::OsStringExt;
use std::{io::Error as IoError, mem};
use std::{ptr, slice};
use winapi::shared::wtypes;
use winapi::um::combaseapi::PropVariantClear;
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

/// RAII objects around `IMMDeviceEnumerator`.
struct Enumerator(*mut IMMDeviceEnumerator);

unsafe impl Send for Enumerator {}
unsafe impl Sync for Enumerator {}

// ---

unsafe fn device_name(device: *mut IMMDevice) -> Result<String, ()> {
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

// TODO: Error handling
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
            let mut device: *mut IMMDevice = ptr::null_mut();
            check_result_empty((*collection).Item(i, &mut device))?;

            let name_string = device_name(device)?;

            println!("{}", name_string);

            let info = AudioDeviceInfo {
                id: DeviceID { name: name_string, unique_id: None },
                in_ports: Vec::new(),     // TODO
                out_ports: Vec::new(),    // TODO
                sample_rates: Vec::new(), // TODO: WASAPI won't tell us directly what sample rates are supported, so we'll need to guess and check - see CPAL source
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
        default_device: None,
    })
}
