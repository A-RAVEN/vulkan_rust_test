
use ash::vk;
use crate::utility::{structs::*, constants::*};

pub struct SwapchainSupportDetails
{
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct SwapchainContext
{
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
}


fn query_swapchain_support(
    physical_device: vk::PhysicalDevice,
    surface_context: &SurfaceContext,
) -> SwapchainSupportDetails
{
    unsafe
    {
        let capabilities = surface_context
            .surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface_context.surface)
            .expect("Vulkan query  surface capabilities failed!");
        let formats = surface_context
            .surface_loader
            .get_physical_device_surface_formats(physical_device, surface_context.surface)
            .expect("Vulkan query surface formats failed!");
        let present_modes = surface_context
            .surface_loader
            .get_physical_device_surface_present_modes(physical_device, surface_context.surface)
            .expect("Vulkan query surface present modes failed!");

            SwapchainSupportDetails { capabilities, formats, present_modes }
    }
}

fn choose_swapchain_format(
    formats_available: &Vec<vk::SurfaceFormatKHR>,
) -> vk::SurfaceFormatKHR
{
    for surface_format in formats_available.iter()
    {
        if surface_format.format == vk::Format::B8G8R8A8_SRGB
            && surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return surface_format.clone();   
        }
    }
    formats_available.first().unwrap().clone()
}

fn choose_swapchain_present_mode(
    present_modes_available: &Vec<vk::PresentModeKHR>,
) -> vk::PresentModeKHR
{
    for &present_mode in present_modes_available.iter()
    {
        if present_mode == vk::PresentModeKHR::MAILBOX {
            return present_mode;
        }
    }
    vk::PresentModeKHR::FIFO
}

fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D 
{
    if capabilities.current_extent.width != u32::MAX
    {
        capabilities.current_extent
    }
    else
    {
        use num::clamp;

        vk::Extent2D{
            width: clamp(
                WINDOW_WIDTH,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: clamp(
                WINDOW_HEIGHT,
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
}

pub fn create_swapchain(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    surface_context: &SurfaceContext,
    queue_family: &QueueFamilyIndices,
) -> SwapchainContext
{
    let swapchain_support_details = query_swapchain_support(physical_device, surface_context);

    let surface_format = choose_swapchain_format(&swapchain_support_details.formats);
    
    let present_mode = choose_swapchain_present_mode(&swapchain_support_details.present_modes);

    let extent = choose_swapchain_extent(&swapchain_support_details.capabilities);

    let image_count = swapchain_support_details.capabilities.min_image_count + 1;

    let image_count = 
        if swapchain_support_details.capabilities.max_image_count > 0
        {image_count.min(swapchain_support_details.capabilities.max_image_count)}
        else{image_count};

    let (image_sharing_mode, queue_family_indices) = 
    {
        if queue_family.graphics_family != queue_family.present_family
        {
            (
                vk::SharingMode::CONCURRENT,
                vec![
                    queue_family.graphics_family.unwrap(),
                    queue_family.present_family.unwrap(),
                ],
            )
        }
        else {
            (
                vk::SharingMode::EXCLUSIVE,
                vec![],
            )
        }
    };

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .flags(vk::SwapchainCreateFlagsKHR::empty())
        .surface(surface_context.surface)
        .min_image_count(image_count)
        .image_color_space(surface_format.color_space)
        .image_format(surface_format.format)
        .image_extent(extent)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(swapchain_support_details.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null())
        .image_array_layers(1);
    
    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);

    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Vulkan swapchain creation failed!")
    };

    let swapchain_images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)
            .expect("Vulkan get swapchain images failed!")
    };

    SwapchainContext 
    { 
        swapchain_loader: swapchain_loader
        , swapchain: swapchain
        , swapchain_images: swapchain_images
        , swapchain_format: surface_format.format
        , swapchain_extent: extent 
    }
}
