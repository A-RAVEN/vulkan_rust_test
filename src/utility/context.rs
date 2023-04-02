use ash::vk;
use crate::utility::{tools, debug, constants::*, structs, platforms, self};

use std::{ffi::{CString, c_char, CStr}, collections::HashSet};



pub fn create_instance(entry: &ash::Entry, window_title: &str, validation_info: &debug::ValidationInfo) -> ash::Instance
{
    if validation_info.enabled && debug::check_validation_layer_support(entry) == false
    {
        panic!("Requested Vulkan Validation Layers Not Available!");
    }

    let app_name = CString::new(window_title).unwrap();
    let engine_name = CString::new("Vulkan Engine").unwrap();
    
    let app_info = vk::ApplicationInfo::builder()
        .application_name(app_name.as_c_str())
        .engine_name(engine_name.as_c_str())
        .application_version(APPLICATION_VERSION)
        .api_version(API_VERSION)
        .engine_version(ENGINE_VERSION);

    // Get Required Extension Names
    let extension_names = platforms::required_extension_names();

    let mut create_info = vk::InstanceCreateInfo::builder()
        .flags(vk::InstanceCreateFlags::empty())
        .application_info(&app_info)
        .enabled_extension_names(&extension_names);



    // Get Required Validation Layers
    let enabled_layer_names_c: Vec<CString> = validation_info
        .required_validation_layers
        .iter()
        .map(|layer_name_c| CString::new(*layer_name_c).unwrap())
        .collect();

    let enabled_layer_names:Vec<*const c_char> = enabled_layer_names_c
        .iter()
        .map(|layer_name_c| layer_name_c.as_ptr())
        .collect();

    if validation_info.enabled 
    {
        create_info = create_info.enabled_layer_names(&enabled_layer_names);
    }

    let instance: ash::Instance = unsafe {
        entry.create_instance(&create_info, None)
            .expect("Failed To Create VK Instance!")
    };

    instance
}

pub fn pick_physical_device(instance: &ash::Instance, surface_contet: &structs::SurfaceContext) -> vk::PhysicalDevice 
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
        (result, device_score) = iterate_select_device(instance, result, device_score, physical_device, surface_contet);
    }

    return result.unwrap();
}

pub fn create_logical_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice, surface_context: &structs::SurfaceContext, validation_info: &debug::ValidationInfo) -> (ash::Device, structs::QueueFamilyIndices)
{
    //TODO We Find Queue Family Twice, Try Only Once
    let indices = find_queue_family_indices(instance, physical_device, surface_context);

    let queue_priorities = [1.0f32];
    let queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .flags(vk::DeviceQueueCreateFlags::empty())
        .queue_family_index(indices.graphics_family.unwrap())
        .queue_priorities(&queue_priorities);
    
    let physical_device_features = vk::PhysicalDeviceFeatures::builder();

    let enabled_layer_names_c: Vec<CString> = validation_info
        .required_validation_layers
        .iter()
        .map(|layer_name_c| CString::new(*layer_name_c).unwrap())
        .collect();

    let enabled_layer_names:Vec<*const c_char> = enabled_layer_names_c
        .iter()
        .map(|layer_name_c| layer_name_c.as_ptr())
        .collect();

    let enabled_extension_names = get_required_device_extension_names();

    let mut device_create_info = vk::DeviceCreateInfo::builder()
        .flags(vk::DeviceCreateFlags::empty())
        .queue_create_infos(std::slice::from_ref(&queue_create_info))
        .enabled_features(&physical_device_features)
        .enabled_extension_names(&enabled_extension_names);

    if validation_info.enabled
    {
        device_create_info = device_create_info.enabled_layer_names(&enabled_layer_names);
    }

    let device: ash::Device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .expect("Vulkan Failed To Create Logical Device!")
    };

    (device, indices)
}

pub fn create_surface(entry: &ash::Entry, instance: &ash::Instance, window: &winit::window::Window) -> structs::SurfaceContext
{
    let surface = unsafe {
        platforms::create_surface(entry, instance, window)
            .expect("Vulkan Failed To Create Surface")
    };

    let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
    structs::SurfaceContext { surface_loader: surface_loader, surface: surface }
}

//Private Functions
fn find_queue_family_indices(instance: &ash::Instance, current_device: vk::PhysicalDevice, surface_contet: &structs::SurfaceContext) -> structs::QueueFamilyIndices 
{
    let queue_families = unsafe {
        instance.get_physical_device_queue_family_properties(current_device)
    };
     
    let mut queue_family_indicecs = structs::QueueFamilyIndices::new();

    let mut index = 0;
    for queue_family_prop in queue_families.iter()
    {
        if queue_family_prop.queue_count > 0
        {
            if queue_family_indicecs.graphics_family.is_none() && queue_family_prop.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indicecs.graphics_family = Some(index);
            }

            if queue_family_indicecs.present_family.is_none()
            {
                let queue_family_support_present = surface_contet.queue_family_supports_present(current_device, index);
                if queue_family_support_present
                {
                    queue_family_indicecs.present_family = Some(index);
                }
            }
        }

        if queue_family_indicecs.is_compelete()
        {
            break;
        }

        index += 1;
    }

    queue_family_indicecs
}

fn iterate_select_device(instance: &ash::Instance, last_device: Option<vk::PhysicalDevice>, last_device_score: u8, current_device: vk::PhysicalDevice, surface_contet: &structs::SurfaceContext) -> (Option<vk::PhysicalDevice>, u8)
{
    let device_properties = unsafe {
        instance.get_physical_device_properties(current_device)
    };
    // let device_features = unsafe {
    //     instance.get_physical_device_features(current_device)
    // };
    // let device_families = unsafe {
    //     instance.get_physical_device_queue_family_properties(current_device)
    // };
    let (device_type_name, device_type_score) = match device_properties.device_type{
        vk::PhysicalDeviceType::DISCRETE_GPU => ("Discrete GPU", 4),
        vk::PhysicalDeviceType::INTEGRATED_GPU => ("Integrate GPU", 3),
        vk::PhysicalDeviceType::VIRTUAL_GPU => ("Virtual GPU", 2),
        vk::PhysicalDeviceType::CPU => ("CPU", 1),
        vk::PhysicalDeviceType::OTHER => ("Unknown", 0),
        _ => panic!("Invalid Physical Device Type"),
    };

    let device_name = tools::char_array_to_string(&device_properties.device_name);
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

    let queue_family_indices = find_queue_family_indices(instance, current_device, surface_contet);

    let support_extensions = is_device_support_extensions(instance, current_device);

    if device_type_score <= 0 || (queue_family_indices.is_compelete() == false) || !support_extensions
    {
        return (last_device, last_device_score);
    }

    return (Some(current_device), device_type_score);
}



fn is_device_support_extensions(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> bool
{
    let available_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)
            .expect("Vulkan Enumerate Physical Device Extension Properties Failed!")
    };
    let mut required_extension_name_set = HashSet::new();
    let require_extension_names = get_required_device_extension_names();
    for required_extension_name in require_extension_names.iter()
    {
        let raw_extension_str = unsafe{
            CStr::from_ptr(*required_extension_name)
        };
        required_extension_name_set.insert(raw_extension_str.to_str().unwrap().to_owned());
    }

    println!("Available Extensions: ");
    for extension in available_extensions.iter()
    {
        let extension_name = utility::tools::char_array_to_string(&extension.extension_name);
        println!("\t Name: {}, Version {}", extension_name, extension.spec_version);
        required_extension_name_set.remove(&extension_name);
    }

    let support = required_extension_name_set.is_empty();

    if !support
    {
        println!("UnSupported Extensions: ");
        for extension_name in required_extension_name_set.iter()
        {
            println!("\t{}", extension_name);
        }
    }

    return support;
}