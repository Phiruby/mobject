use crate::c_utils::Utf8Pointer;
use ash::{
    Device, Instance,
    khr::surface,
    vk::{
        self, DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceFeatures,
        PhysicalDeviceFeatures2, PhysicalDeviceProperties, QueueFamilyProperties2, StructureType,
        SurfaceKHR,
    },
};
use dialoguer::FuzzySelect;
use std::{ffi::CStr, fmt::Debug};

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

pub struct QueueFamilies {
    pub graphics_index: usize,
    pub presentation_index: usize,
}

fn get_queue_index_with_capability(
    capability: vk::QueueFlags,
    properties: &Vec<QueueFamilyProperties2>,
) -> usize {
    let mut potential_queues = properties
        .iter()
        .enumerate()
        .filter(|(_, queue)| {
            queue
                .queue_family_properties
                .queue_flags
                .intersects(capability)
        })
        .map(|(index, _)| index);

    match potential_queues.next() {
        Some(x) => x,
        _ => panic!("No queue found with capability {:?}", capability),
    }
}

fn get_presentation_queue_index(
    properties: &Vec<QueueFamilyProperties2>,
    surface_instance: &surface::Instance,
    surface: SurfaceKHR,
    physical_device: PhysicalDevice,
) -> usize {
    properties
        .iter()
        .enumerate()
        .filter(|(queue_family_index, family_properties)| {
            unsafe {
                surface_instance.get_physical_device_surface_support(
                    physical_device,
                    *queue_family_index as u32,
                    surface,
                )
            }
            .unwrap()
        })
        .map(|(idx, _)| idx)
        .next()
        .expect("No presentation queue family found!")
}

impl QueueFamilies {
    pub fn new(
        instance: &Instance,
        surface_instance: &surface::Instance,
        surface: SurfaceKHR,
        physical_device: PhysicalDevice,
    ) -> Self {
        let num_queues =
            unsafe { instance.get_physical_device_queue_family_properties2_len(physical_device) };

        let mut queue_families: Vec<QueueFamilyProperties2> =
            vec![QueueFamilyProperties2::default(); num_queues];
        unsafe {
            instance
                .get_physical_device_queue_family_properties2(physical_device, &mut queue_families)
        };
        Self {
            graphics_index: get_queue_index_with_capability(
                vk::QueueFlags::GRAPHICS,
                &queue_families,
            ),
            presentation_index: get_presentation_queue_index(
                &queue_families,
                surface_instance,
                surface,
                physical_device,
            ),
        }
    }
}

pub fn create_logical_device<S: AsRef<str> + Debug>(
    queues: &QueueFamilies,
    instance: &Instance,
    device: PhysicalDevice,
    physical_device_features: Option<PhysicalDeviceFeatures>,
    extension_names: Option<Vec<S>>,
) -> Device {
    let device_features = match physical_device_features {
        Some(x) => x,
        _ => PhysicalDeviceFeatures::default(),
    };

    let device_extensions = match extension_names {
        Some(x) => x,
        _ => Vec::new(),
    };
    let device_extension_ptrs = Utf8Pointer::new(&device_extensions);
    let p: f32 = 0.0;
    let queue_create_infos = vec![DeviceQueueCreateInfo {
        s_type: StructureType::DEVICE_QUEUE_CREATE_INFO,
        queue_family_index: queues.graphics_index as u32,
        queue_count: 1,
        p_queue_priorities: &p,
        ..Default::default()
    }];

    let device_create_info = DeviceCreateInfo {
        s_type: StructureType::DEVICE_CREATE_INFO,
        p_queue_create_infos: queue_create_infos.as_ptr(),
        queue_create_info_count: queue_create_infos.len() as u32,
        pp_enabled_extension_names: device_extension_ptrs.as_ptr(),
        enabled_extension_count: device_extensions.len() as u32,
        p_enabled_features: &device_features,
        ..Default::default()
    };

    unsafe { instance.create_device(device, &device_create_info, None) }.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoketest_physical_device() {}
}
