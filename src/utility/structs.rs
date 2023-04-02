

use ash::vk;
pub struct QueueFamilyIndices
{
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices 
{
    pub fn new() -> QueueFamilyIndices
    {
        QueueFamilyIndices{
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn is_compelete(&self) -> bool
    {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct DeviceExtensionInfo
{
    pub names: [& 'static str; 1],
}

pub struct SurfaceContext
{
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,
}

impl SurfaceContext
{
    pub fn queue_family_supports_present(&self, physical_device: vk::PhysicalDevice, queue_family_index: u32) -> bool
    {
        unsafe
        {
            self.surface_loader
                .get_physical_device_surface_support(physical_device, queue_family_index, self.surface)
                .unwrap()
        }
    }
}