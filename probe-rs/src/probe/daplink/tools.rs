use std::time::Duration;
use rusb::{Device, UsbContext};
use crate::probe::{DebugProbeInfo, DebugProbeType};
use super::DAPLinkDevice;

/// Finds all CMSIS-DAP devices, either v1 (HID) or v2 (WinUSB Bulk).
///
/// This method uses rusb to read device strings, which might fail due
/// to permission or driver errors, so it falls back to listing only
/// HID devices if it does not find any suitable devices.
pub fn list_daplink_devices() -> Vec<DebugProbeInfo> {
    match rusb::Context::new().and_then(|ctx| ctx.devices()) {
        Ok(devices) => devices.iter().filter_map(get_daplink_info).collect(),
        Err(_) => match hidapi::HidApi::new() {
            Ok(api) => api.device_list().filter_map(get_daplink_hid_info).collect(),
            Err(_) => vec![],
        },
    }
}

/// Checks if a given Device is a CMSIS-DAP probe, returning Some(DebugProbeInfo) if so.
fn get_daplink_info(device: Device<rusb::Context>) -> Option<DebugProbeInfo> {
    // Open device handle and read basic information
    let timeout = Duration::from_millis(100);
    let d_desc = device.device_descriptor().ok()?;
    let handle = device.open().ok()?;
    let language = handle.read_languages(timeout).ok()?[0];
    let prod_str = handle.read_product_string(language, &d_desc, timeout).ok()?;
    let sn_str = handle.read_serial_number_string(language, &d_desc, timeout).ok();

    // All CMSIS-DAP probes must have "CMSIS-DAP" in their product string.
    if prod_str.contains("CMSIS-DAP") {
        Some(DebugProbeInfo {
            identifier: prod_str,
            vendor_id: d_desc.vendor_id(),
            product_id: d_desc.product_id(),
            serial_number: sn_str,
            probe_type: DebugProbeType::DAPLink,
        })
    } else {
        None
    }
}

/// Checks if a given HID device is a CMSIS-DAP v1 probe, returning Some(DebugProbeInfo) if so.
fn get_daplink_hid_info(device: &hidapi::DeviceInfo) -> Option<DebugProbeInfo> {
    if let Some(prod_str) = device.product_string() {
        if prod_str.contains("CMSIS-DAP") {
            return Some(DebugProbeInfo {
                identifier: prod_str.to_owned(),
                vendor_id: device.vendor_id(),
                product_id: device.product_id(),
                serial_number: device.serial_number().map(|s| s.to_owned()),
                probe_type: DebugProbeType::DAPLink,
            });
        }
    }
    None
}

/// Attempt to open the given device in CMSIS-DAP v2 mode
pub fn open_v2_device(device: Device<rusb::Context>) -> Option<DAPLinkDevice> {
    // Open device handle and read basic information
    let timeout = Duration::from_millis(100);
    let d_desc = device.device_descriptor().ok()?;
    let vid = d_desc.vendor_id();
    let pid = d_desc.product_id();
    let mut handle = device.open().ok()?;
    let language = handle.read_languages(timeout).ok()?[0];

    // Go through interfaces to try and find a v2 interface.
    // The CMSIS-DAPv2 spec says that v2 interfaces should use a specific
    // WinUSB interface GUID, but in addition to being hard to read, the
    // official DAPlink firmware doesn't use it. Instead, we scan for an
    // interface whose string contains "CMSIS-DAP" and has two or three
    // endpoints of the correct type and direction.
    let c_desc = device.config_descriptor(0).ok()?;
    for interface in c_desc.interfaces() {
        for i_desc in interface.descriptors() {
            // Skip interfaces without "CMSIS-DAP" in their string
            match handle.read_interface_string(language, &i_desc, timeout) {
                Ok(i_str) if !i_str.contains("CMSIS-DAP") => continue,
                Err(_) => continue,
                Ok(_) => (),
            }

            // Skip interfaces without 2 or 3 endpoints
            let n_ep = i_desc.num_endpoints();
            if n_ep < 2 || n_ep > 3 {
                continue;
            }

            let eps: Vec<_> = i_desc.endpoint_descriptors().collect();

            // Check the first interface is bulk out
            if eps[0].transfer_type() != rusb::TransferType::Bulk ||
               eps[0].direction()     != rusb::Direction::Out
            {
                continue;
            }

            // Check the second interface is bulk in
            if eps[1].transfer_type() != rusb::TransferType::Bulk ||
               eps[1].direction()     != rusb::Direction::In
            {
                continue;
            }

            // Store EP addresses of the in and out EPs
            let out_ep = eps[0].address();
            let in_ep = eps[1].address();

            // Attempt to claim this interface
            match handle.claim_interface(interface.number()) {
                Ok(()) => {
                    log::debug!("Opening {:04x}:{:04x} in CMSIS-DAPv2 mode", vid, pid);
                    return Some(DAPLinkDevice::V2 {handle, out_ep, in_ep});
                },
                Err(_) => continue,
            }
        }
    }

    // Could not open in v2
    log::debug!("Could not open {:04x}:{:04x} in CMSIS-DAP v2 mode", vid, pid);
    None
}

/// Attempt to open the given DebugProbeInfo in CMSIS-DAP v2 mode if possible,
/// otherwise in v1 mode.
pub fn open_device_from_info(info: &DebugProbeInfo) -> Option<DAPLinkDevice> {
    // Try using rusb to open a v2 device. This might fail if
    // the device does not support v2 operation or due to driver
    // or permission issues with opening bulk devices.
    if let Ok(devices) = rusb::Context::new().and_then(|ctx| ctx.devices()) {
        for device in devices.iter() {
            let d_desc = match device.device_descriptor() {
                Ok(d_desc) => d_desc,
                Err(_) => continue,
            };
            let handle = match device.open() {
                Ok(handle) => handle,
                Err(_) => continue,
            };
            let sn_str = handle.read_serial_number_string_ascii(&d_desc).ok();
            if d_desc.vendor_id() == info.vendor_id &&
               d_desc.product_id() == info.product_id &&
               sn_str == info.serial_number
            {
                // If the VID, PID, and potentially SN all match,
                // attempt to open the device in v2 mode.
                if let Some(device) = open_v2_device(device) {
                    return Some(device);
                }
            }
        }
    }

    // If rusb failed or the device didn't support v2, try using hidapi to open in v1 mode.
    // If this doesn't work we give up and return None.
    let vid = info.vendor_id;
    let pid = info.product_id;
    let sn = &info.serial_number;
    log::debug!("Attempting to open {:04x}:{:04x} in CMSIS-DAP v1 mode", vid, pid);
    match sn {
        Some(sn) => hidapi::HidApi::new().and_then(|api| api.open_serial(vid, pid, &sn)),
        None     => hidapi::HidApi::new().and_then(|api| api.open(vid, pid)),
    }.map(|device| DAPLinkDevice::V1(device)).ok()
}
