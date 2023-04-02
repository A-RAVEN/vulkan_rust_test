use winapi::ctypes::c_char;
use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;

use ash::vk;
//use ash::vk::{DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT};

//use std::collections::btree_map::Iter;
use std::ffi::{CString};
//use std::ptr;

use vulkan_rust_test::{utility, utility::constants::*, utility::tools::*, utility::debug::*};

const WINDOW_TITLE: &'static str = "03.Physical Device Selection";

struct QueueFamilyIndices
{
    graphics_family: Option<u32>,
}

impl QueueFamilyIndices 
{
    pub fn is_compelete(&self) -> bool
    {
        self.graphics_family.is_some()
    }
}

fn find_queue_family_indices(instance: &ash::Instance, current_device: vk::PhysicalDevice) -> QueueFamilyIndices 
{
    let queue_families = unsafe {
        instance.get_physical_device_queue_family_properties(current_device)
    };
     
    let mut queue_family_indicecs = QueueFamilyIndices{
        graphics_family: None
    };

    let mut index = 0;
    for queue_family_prop in queue_families.iter()
    {
        if queue_family_prop.queue_count > 0
            && queue_family_prop.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        {
            queue_family_indicecs.graphics_family = Some(index);
        }

        if queue_family_indicecs.is_compelete()
        {
            break;
        }

        index += 1;
    }

    queue_family_indicecs
}

fn iterate_select_device(instance: &ash::Instance, last_device: Option<vk::PhysicalDevice>, last_device_score: u8, current_device: vk::PhysicalDevice) -> (Option<vk::PhysicalDevice>, u8)
{
    let device_properties = unsafe {
        instance.get_physical_device_properties(current_device)
    };
    let device_features = unsafe {
        instance.get_physical_device_features(current_device)
    };
    let device_families = unsafe {
        instance.get_physical_device_queue_family_properties(current_device)
    };
    let (device_type_name, device_type_score) = match device_properties.device_type{
        vk::PhysicalDeviceType::DISCRETE_GPU => ("Discrete GPU", 4),
        vk::PhysicalDeviceType::INTEGRATED_GPU => ("Integrate GPU", 3),
        vk::PhysicalDeviceType::VIRTUAL_GPU => ("Virtual GPU", 2),
        vk::PhysicalDeviceType::CPU => ("CPU", 1),
        vk::PhysicalDeviceType::OTHER => ("Unknown", 0),
        _ => panic!("Invalid Physical Device Type"),
    };

    let device_name = utility::tools::char_array_to_string(&device_properties.device_name);
    println!(
        "\tDevice Name: {}, id: {}, type: {}",
        device_name, device_properties.device_id, device_type_name
    );

    let major_version = ash::vk::api_version_major(device_properties.api_version);
    let minor_version = ash::vk::api_version_minor(device_properties.api_version);
    let patch_version = ash::vk::api_version_patch(device_properties.api_version);

    println!(
        "\tAPI Version: {}.{}.{}",
        major_version, minor_version, patch_version
    );

    let queue_family_indices = find_queue_family_indices(instance, current_device);

    if device_type_score <= 0 || (queue_family_indices.is_compelete() == false)
    {
        return (last_device, last_device_score);
    }

    return (Some(current_device), device_type_score);
}

fn pick_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice 
{
    let physical_devices = unsafe{
        instance
            .enumerate_physical_devices()
            .expect("Vulkan Failed To Enumerate Physical Devices!")
    };

    println!(
        "{} devices found with vulkan support.",
        physical_devices.len()
    );

    let mut result = None;
    let mut device_score: u8 = 0;
    for &physical_device in physical_devices.iter()
    {
        (result, device_score) = iterate_select_device(instance, result, device_score, physical_device);
    }

    return result.unwrap();
}

struct VulkanApp
{
    _entry: ash::Entry,
    instance: ash::Instance,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    _physical_device: vk::PhysicalDevice,
}

impl VulkanApp
{

    pub fn new() -> VulkanApp
    {
        let entry = unsafe{
            ash::Entry::load().expect("Load Vulkan Entry Fail!")
        };
        let instance = VulkanApp::create_instance(&entry);
        
        let (debug_utils_loader, debug_messenger) = setup_debug_utils(&entry, &instance);

        let physical_device = pick_physical_device(&instance);

        VulkanApp 
        { 
            _entry: entry, 
            instance,
            debug_utils_loader,
            debug_messenger,
            _physical_device: physical_device,
        }
    }

    fn create_instance(entry: &ash::Entry) -> ash::Instance
    {
        if VALIDATION.enabled && check_validation_layer_support(entry) == false
        {
            panic!("Requested Vulkan Validation Layers Not Available!");
        }

        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("Vulkan Engine").unwrap();
        
        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name.as_c_str())
            .engine_name(engine_name.as_c_str())
            .application_version(APPLICATION_VERSION)
            .api_version(API_VERSION)
            .engine_version(ENGINE_VERSION);

        // Get Required Extension Names
        let extension_names = utility::platforms::required_extension_names();

        let mut create_info = vk::InstanceCreateInfo::builder()
            .flags(vk::InstanceCreateFlags::empty())
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);



        // Get Required Validation Layers
        let enabled_layer_names_c: Vec<CString> = VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name_c| CString::new(*layer_name_c).unwrap())
            .collect();

        let enabled_layer_names:Vec<*const c_char> = enabled_layer_names_c
            .iter()
            .map(|layer_name_c| layer_name_c.as_ptr())
            .collect();

        if VALIDATION.enabled 
        {
            create_info = create_info.enabled_layer_names(&enabled_layer_names);
        }

        let instance: ash::Instance = unsafe {
            entry.create_instance(&create_info, None)
                .expect("Failed To Create VK Instance!")
        };

        instance
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

    let vulkan_app = VulkanApp::new();
    vulkan_app.main_loop(event_loop, window);
}