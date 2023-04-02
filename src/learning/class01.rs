use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;

use ash::vk;
use std::ffi::CString;
//use std::ptr;

use vulkan_rust_test::utility::constants::*;
use vulkan_rust_test::utility;

const WINDOW_TITLE: &'static str = "01.Instance Creation";
//const WINDOW_WIDTH: u32 = 800;
//const WINDOW_HEIGHT: u32 = 600;

struct VulkanApp
{
    _entry: ash::Entry,
    instance: ash::Instance,
}

impl VulkanApp
{

    pub fn new() -> VulkanApp
    {
        let entry = unsafe{
            ash::Entry::load().expect("Load Vulkan Entry Fail!")
        };
        let instance = VulkanApp::create_instance(&entry);

        VulkanApp { _entry: entry, instance }
    }

    fn create_instance(entry: &ash::Entry) -> ash::Instance
    {
        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("Vulkan Engine").unwrap();
        
        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name.as_c_str())
            .engine_name(engine_name.as_c_str())
            .application_version(APPLICATION_VERSION)
            .api_version(API_VERSION)
            .engine_version(ENGINE_VERSION);

        let extension_names = utility::platforms::required_extension_names();

        let create_info = vk::InstanceCreateInfo::builder()
            .flags(vk::InstanceCreateFlags::empty())
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

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