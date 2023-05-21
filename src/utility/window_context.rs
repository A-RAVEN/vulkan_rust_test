use winit::dpi::PhysicalSize;
use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;

pub trait WindowLoopContext {
    fn on_resize(&mut self, new_size: PhysicalSize<u32>) {
    }
    fn on_drawframe(&mut self){

    }
}

pub struct WindowContext{
    pub window: winit::window::Window,
    event_loop: EventLoop<()>,
}

impl WindowContext{
    pub fn new(title: &str, width: u32, height: u32) -> WindowContext
    {
        let event_loop = EventLoop::new();
        let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .with_inner_size(winit::dpi::LogicalSize::new(width, height))
        .build(&event_loop)
        .expect("Failed to create Window.");
        WindowContext{
            window,
            event_loop,
        }
    }

    pub fn main_loop<T: WindowLoopContext + 'static>(self, mut context: T)
    {
        self.event_loop.run(move |event, _, control_flow| {
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
                        //窗口缩放
                        | WindowEvent::Resized(new_size) =>{
                            context.on_resize(new_size);
                        }
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
                    self.window.request_redraw();
                },
                | Event::RedrawRequested(_window_id) =>
                {
                    context.on_drawframe();
                }
                | _ => {},
            }
        })
    }
}

pub fn init_window(event_loop: &EventLoop<()>, title: &str, width: u32, height: u32) -> winit::window::Window
{
    winit::window::WindowBuilder::new()
        .with_title(title)
        .with_inner_size(winit::dpi::LogicalSize::new(width, height))
        .build(event_loop)
        .expect("Failed to create Window.")
}

pub fn main_loop<T: WindowLoopContext + 'static>(event_loop: EventLoop<()>, mut context: T, window: Window)
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
                    //窗口缩放
                    | WindowEvent::Resized(new_size) =>{
                        context.on_resize(new_size);
                    }
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
                context.on_drawframe();
            }
            | _ => {},
        }
    })
}