pub mod c_utils;
pub mod device;
use ash::vk::{self, ApplicationInfo, InstanceCreateInfo, StructureType};
use ash::{Entry, Instance};
use c_utils::Utf8Pointer;
use std::ffi::CString;
const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub struct Scene {}

impl Scene {
    pub fn new() {
        let entry = unsafe { Entry::load().unwrap() };
        let instance = create_vk_instance(&entry);
        let physical_device = device::select_physical_device(&instance);
    }
}

fn create_vk_instance(entry: &Entry) -> Instance {
    let app_name = CString::new("Mobject").unwrap();
    let engine_name = CString::new("No Engine").unwrap();

    let app_info = ApplicationInfo {
        s_type: StructureType::APPLICATION_INFO,
        p_application_name: app_name.as_ptr(),
        application_version: vk::make_api_version(1, 0, 0, 0),
        p_engine_name: engine_name.as_ptr(),
        engine_version: vk::make_api_version(1, 1, 1, 0),
        api_version: vk::make_api_version(0, 1, 1, 0),
        ..Default::default()
    };

    #[cfg(feature = "validation_layers")]
    let utf8_ptr = Utf8Pointer::new(&VALIDATION_LAYERS);

    let instance_info = InstanceCreateInfo {
        s_type: StructureType::INSTANCE_CREATE_INFO,
        p_application_info: &app_info,
        #[cfg(feature = "validation_layers")]
        enabled_layer_count: 1,
        #[cfg(feature = "validation_layers")]
        pp_enabled_layer_names: utf8_ptr.as_ptr(),
        ..Default::default()
    };

    unsafe { entry.create_instance(&instance_info, None).unwrap() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoketest_it_works() {
        Scene::new();
    }
}
