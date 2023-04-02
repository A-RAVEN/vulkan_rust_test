use ash::vk::make_api_version;

use crate::utility::{debug::ValidationInfo, structs::*};

pub const APPLICATION_VERSION: u32 = make_api_version(0, 1, 0, 0);
pub const ENGINE_VERSION: u32 = make_api_version(0, 1, 0, 0);
pub const API_VERSION: u32 =  ash::vk::API_VERSION_1_3;// make_api_version(0,  1, 0, 92);

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub const IS_PAINT_FPS_COUNTER: bool = false;

pub const VALIDATION: ValidationInfo = ValidationInfo
{
    enabled: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

pub fn get_required_device_extension_names() -> Vec<* const i8>
{
    vec![
        ash::extensions::khr::Swapchain::name().as_ptr(),
    ]
}