use ash::{
    Instance,
    vk::{PhysicalDevice, PhysicalDeviceProperties},
};
use dialoguer::FuzzySelect;
use std::ffi::CStr;

pub fn select_physical_device(instance: &Instance) -> PhysicalDevice {
    let physical_devices = unsafe { instance.enumerate_physical_devices() }.unwrap();

    if physical_devices.len() == 1 {
        return physical_devices[0];
    }
    let device_properties: Vec<PhysicalDeviceProperties> = physical_devices
        .iter()
        .copied()
        .map(|pd| unsafe { instance.get_physical_device_properties(pd) })
        .collect();

    let options: Vec<String> = device_properties
        .iter()
        .map(|dp| {
            let device_name = dp.device_name.as_ptr();
            let cstr = unsafe { CStr::from_ptr(device_name) };
            cstr.to_string_lossy().into_owned()
        })
        .collect();

    let chosen_device = FuzzySelect::new()
        .with_prompt("Choose your device")
        .items(&options)
        .interact()
        .unwrap();
    physical_devices[chosen_device]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoketest_physical_device() {}
}
