//use winapi::ctypes::c_char;
use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;

use ash::vk;

use vulkan_rust_test::utility::{constants::*, debug::*, structs::*, context::*, swapchain::*};

const WINDOW_TITLE: &'static str = "07.Swapchain ImageView";

use std::fs;


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
    swapchain_image_views: Vec<vk::ImageView>,
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

        let swapchain_image_views = VulkanApp::create_image_views_2d(
            &logical_device,
            swapchain_context.swapchain_format,
            &swapchain_context.swapchain_images);

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
            swapchain_image_views: swapchain_image_views,
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

    pub fn create_image_views_2d(
        logical_device: &ash::Device,
        image_format: vk::Format,
        images: &Vec<vk::Image>,
    ) -> Vec<vk::ImageView>
    {
        let mut swapchain_image_views = vec![];

        for &image in images.iter()
        {
            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .flags(vk::ImageViewCreateFlags::empty())
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(image_format)
                .components(vk::ComponentMapping{
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange{
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image(image);

            let image_view = unsafe{
                logical_device
                    .create_image_view(&image_view_create_info, None)
                    .expect("Vulkan Failed To Create Image View")
            };

            swapchain_image_views.push(image_view);
        }

        swapchain_image_views
    }

}

impl Drop for VulkanApp
{
    fn drop(&mut self)
    {
        unsafe
        {
            for &image_view in self.swapchain_image_views.iter()
            {
                self.device.destroy_image_view(image_view, None);
            }
            self.swapchain_context.swapchain_loader.destroy_swapchain(self.swapchain_context.swapchain, None);
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

fn compute(input: &mut u32, output: &mut u32) {
    let mut tmp = *output;
    if *input > 10 {
        tmp = 1;
    }
    if *input > 5 {
        tmp *= 2;
    }
    *output = tmp;
    // remember that `output` will be `2` if `input > 10`
}

fn main() {

    let mut num = 15;
    //compute(&mut num, &mut num);

    let contents = fs::read_to_string("testStr.txt");
    println!("Text: {0}", contents.unwrap());

    let event_loop = EventLoop::new();
    let window = VulkanApp::init_window(&event_loop);

    let vulkan_app = VulkanApp::new(&window);
    vulkan_app.main_loop(event_loop, window);
}