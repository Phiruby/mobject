pub mod c_utils;

use ash::vk::{self, ApplicationInfo, InstanceCreateInfo, StructureType};
use crate::c_utils::Utf8Pointer;

const VALIDATION_LAYERS: Vec<&'static str> = [
    "VK_LAYER_KHRONOS_validation"
];

pub struct Scene {}

fn create_vk_instance() {
    let app_info = ApplicationInfo {
        s_type: StructureType::APPLICATION_INFO,
        p_application_name: "Mobject",
        application_version: vk::make_version(1.0, 0, 0),
        p_engine_name: "No Engine",
        engine_version: vk::make_version(1, 1, 1),
        api_version: vk::make_api_version(0, 1, 1, 0),
        ....Default::default()
    };
    
    #[cfg(feature="validation_layers")]
    let utf8_ptr = Utf8Pointer::new(VALIDATION_LAYERS);

    let instance_info = InstanceCreateInfo {
        s_type: StructureType::INSTANCE_CREATE_INFO,
        p_application_info: &app_info,
        #[cfg(feature="validation_layers")]
        enabled_layer_count: 1,
        #[cfg(feature="validation_layers")]
        pp_enabled_layer_names: utf8_ptr.as_ptr(),
        ..Default::default()
    };
}
