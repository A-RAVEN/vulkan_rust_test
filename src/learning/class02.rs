use winapi::ctypes::c_char;
use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;

use ash::vk;
use ash::vk::{DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT};

use std::ffi::{CString, CStr, c_void};
use std::ptr;

use vulkan_rust_test::{utility, utility::constants::*, utility::tools::*, utility::debug::ValidationInfo};

const WINDOW_TITLE: &'static str = "02.Validation Layers";

//Shadowing, Moving To debug module
const VALIDATION: ValidationInfo = ValidationInfo
{
    enabled: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

//Moving To debug module later
unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32
{
    let severity = match message_severity
    {
        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };

    let types = match message_type
    {
        DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING => "[DeviceBinding]",
        _ => "[Unknown]",
    };

    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Vulkan Log]{}{}{:?}", severity, types, message);

    vk::FALSE
}

//Moving To debug module later
fn populate_debug_messenger_create_info<'a>() -> vk::DebugUtilsMessengerCreateInfoEXTBuilder<'a>
{
    vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .flags(vk::DebugUtilsMessengerCreateFlagsEXT::empty())
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
        )
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback))
        .user_data(ptr::null_mut())
}

//Moving To debug module later
///return true if supports validation layer
fn check_validation_layer_support(entry: &ash::Entry) -> bool
{
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate Instance Layers Properties!");

    if layer_properties.len() <= 0
    {
        println!("No Available Instance Layers!");
        return false;
    }
    else
    {
        println!("Available Instance Layers:");
        for layer in layer_properties.iter()
        {
            let layer_name = char_array_to_string(&layer.layer_name);
            println!("\t{}", layer_name);
        }
    }

    for required_layer_name in VALIDATION.required_validation_layers.iter()
    {
        let mut layer_found = false;

        for layer_property in layer_properties.iter()
        {
            let test_layer_name = char_array_to_string(&layer_property.layer_name);
            if(*required_layer_name) == test_layer_name
            {
                layer_found = true;
                break;
            }
        }

        if layer_found == false
        {
            return false;
        }
    } 

    return true;
}

//Moving To debug module later
fn setup_debug_utils(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> (ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT)
{
    let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

    if VALIDATION.enabled == false
    {
        return (debug_utils_loader, ash::vk::DebugUtilsMessengerEXT::null());
    }
    else 
    {
        let messenger_create_info = populate_debug_messenger_create_info();
        
        let utils_messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&messenger_create_info, None)
                .expect("Vulkan Debug Utils Messenger Creation Failed")
        };
        return (debug_utils_loader, utils_messenger);
    }
}

struct VulkanApp
{
    _entry: ash::Entry,
    instance: ash::Instance,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
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

        VulkanApp 
        { 
            _entry: entry, 
            instance,
            debug_utils_loader,
            debug_messenger,
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