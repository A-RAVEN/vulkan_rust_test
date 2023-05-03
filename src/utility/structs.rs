

use ash::vk;
use crate::utility::{constants};
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct DeviceExtensionInfo
{
    pub names: [& 'static str; 1],
}

#[derive(Clone)]
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


#[derive(Debug, Clone)]
pub struct FrameSyncContext
{
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
}

impl FrameSyncContext
{
    pub fn destroy_context(&mut self, device: &ash::Device)
    {
        unsafe
        {
            for &semaphore in self.image_available_semaphores.iter()
            {
                device
                    .destroy_semaphore(semaphore, None);
            }
            for &semaphore in self.render_finished_semaphores.iter()
            {
                device
                    .destroy_semaphore(semaphore, None);
            }
            for &fence in self.in_flight_fences.iter()
            {
                device
                    .destroy_fence(fence, None);
            }
        };
    }

    pub fn new(device: &ash::Device) -> FrameSyncContext{
        let mut sync_context = FrameSyncContext{
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            in_flight_fences: vec![]
        };
    
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
    
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);
    
        for _ in 0..constants::MAX_FRAMES_IN_FLIGHT{
            unsafe{
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed To Create Semaphore");
    
                let render_finish_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed To Create Semaphore");
    
                let in_flight_fence = device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed To Create Fence");
    
                sync_context.image_available_semaphores.push(image_available_semaphore);
                sync_context.render_finished_semaphores.push(render_finish_semaphore);
                sync_context.in_flight_fences.push(in_flight_fence);
            }
        }
    
        sync_context
    }
}