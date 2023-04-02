

use ash::vk;
use ash::vk::{DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT};
use crate::utility::{tools::char_array_to_string, constants::VALIDATION};

use std::ffi::{CStr, c_void};
use std::ptr;
//ValidationInfo For

pub struct ValidationInfo
{
    pub enabled: bool,
    pub required_validation_layers: [&'static str; 1],
}

pub unsafe extern "system" fn vulkan_debug_utils_callback(
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

pub fn populate_debug_messenger_create_info<'a>() -> vk::DebugUtilsMessengerCreateInfoEXTBuilder<'a>
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
            //| vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            //| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback))
        .user_data(ptr::null_mut())
}

///return true if supports validation layer
pub fn check_validation_layer_support(entry: &ash::Entry) -> bool
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

pub fn setup_debug_utils(
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