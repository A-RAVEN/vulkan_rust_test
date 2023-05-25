use gpu_allocator::d3d12::AllocatorCreateDesc;

use glam::{Vec2, Vec3};
use vulkan_rust_test::vulkan_application::{vulkan_app::*};

use ash::vk::{self, PipelineShaderStageCreateFlags, ShaderStageFlags
    , PrimitiveTopology, Offset2D, CullModeFlags, FrontFace
    , PolygonMode, SampleCountFlags, AttachmentReference
    , CommandBufferUsageFlags, CommandPoolResetFlags, CommandPool, CommandBufferLevel};

use vulkan_rust_test::utility::{constants::*
    , debug::*, structs::*, context::*, swapchain::*
    , file_system::*, commandbuffers::*, gpubuffer::*, window_context::*};

const WINDOW_TITLE: &'static str = "10. Hello Triangle!";

use std::fs;
use std::ffi::CString;
use memoffset::offset_of;
use gpu_allocator::vulkan;

use winit::event_loop::{EventLoop, ControlFlow};


fn main() {

    let window_context = WindowContext::new(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    let vulkan_app = VulkanApp::new(&window_context.window, WINDOW_TITLE);
    window_context.main_loop(vulkan_app);
}