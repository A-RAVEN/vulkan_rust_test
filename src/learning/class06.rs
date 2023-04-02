//use winapi::ctypes::c_char;
use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;

use ash::vk;
//use ash::vk::{DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT};

//use std::collections::btree_map::Iter;
//use std::ffi::{CString};
//use std::ptr;

use vulkan_rust_test::{utility, utility::constants::*, utility::debug::*, utility::structs::*, utility::context::*};

const WINDOW_TITLE: &'static str = "06.Swapchain Creation";

struct SwapchainContext
{
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
}

struct SwapchainSupportDetails
{
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

struct VulkanApp
{
    _entry: ash::Entry,
    instance: ash::Instance,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    _physical_device: vk::PhysicalDevice,
    device: ash::Device,
    _graphics_queue: vk::Queue,
    _present_queue: vk::Queue,
    surface_context: SurfaceContext,

    swapchain_context: SwapchainContext,
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

fn create_swapchain(
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

impl VulkanApp
{

    pub fn new(window: &winit::window::Window) -> VulkanApp
    {
        let entry = unsafe{
            ash::Entry::load().expect("Load Vulkan Entry Fail!")
        };
        let instance = create_instance(&entry, WINDOW_TITLE, &VALIDATION);
        
        let (debug_utils_loader, debug_messenger) = setup_debug_utils(&entry, &instance);

        let surface_context = create_surface(&entry, &instance, window);

        let physical_device = pick_physical_device(&instance, &surface_context);

        let (logical_device, queue_family_indices) = create_logical_device(&instance, physical_device, &surface_context, &VALIDATION);

        let graphics_queue = unsafe {
            logical_device.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0)
        };

        let present_queue = unsafe {
            logical_device.get_device_queue(queue_family_indices.present_family.unwrap(), 0)
        };

        let swapchain_context = create_swapchain(
            &instance, 
            &logical_device, physical_device, 
            &surface_context, 
            &queue_family_indices);

        VulkanApp 
        { 
            _entry: entry, 
            instance,
            debug_utils_loader,
            debug_messenger,
            _physical_device: physical_device,
            device: logical_device,
            _graphics_queue: graphics_queue,
            _present_queue: present_queue,
            surface_context,
            swapchain_context,
        }
    }

    fn draw_frame(&mut self)
    {
        // Do Drawing
    }

    fn init_window(event_loop: &EventLoop<()>) -> winit::window::Window
    {
        winit::window::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .expect("Failed to create Window.")
    }

    pub fn main_loop(mut self, event_loop: EventLoop<()>, window: Window)
    {
        event_loop.run(move |event, _, control_flow| {
            match event {
                | Event::WindowEvent { event, ..} =>
                {
                    match event
                    {
                        //关闭
                        | WindowEvent::CloseRequested=>
                        {
                            *control_flow = ControlFlow::Exit;
                        },
                        //键盘输入
                        | WindowEvent::KeyboardInput {input, .. } =>{
                            match input 
                            {
                                | KeyboardInput {virtual_keycode, state, .. } =>
                                {
                                    match (virtual_keycode, state)
                                    {
                                        | (Some(VirtualKeyCode::Escape), ElementState::Pressed) => 
                                        {
                                            dbg!();
                                            *control_flow = ControlFlow::Exit;
                                        },
                                        | _ => {},
                                    }
                                },
                            }
                        },
                        //相当于default
                        | _ => {},
                    }
                },
                | Event::MainEventsCleared => 
                {
                    window.request_redraw();
                },
                | Event::RedrawRequested(_window_id) =>
                {
                    self.draw_frame();
                }
                | _ => {},
            }
        })
    }
}

impl Drop for VulkanApp
{
    fn drop(&mut self)
    {
        unsafe
        {
            self.device.destroy_device(None);
            self.surface_context.surface_loader.destroy_surface(self.surface_context.surface, None);
            if VALIDATION.enabled
            {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = VulkanApp::init_window(&event_loop);

    let vulkan_app = VulkanApp::new(&window);
    vulkan_app.main_loop(event_loop, window);
}