use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};

const WINDOW_TITLE: &'static str = "00.Base Code";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

struct VulkanApp;

impl VulkanApp
{
    fn init_window(event_loop: &EventLoop<()>) -> winit::window::Window
    {
        winit::window::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .expect("Failed to create Window.")
    }

    pub fn main_loop(event_loop: EventLoop<()>)
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
                | _ => {},
            }
        })
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let _window = VulkanApp::init_window(&event_loop);

    VulkanApp::main_loop(event_loop);
}