//use winapi::ctypes::c_char;
use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;

use ash::vk::{self, PipelineShaderStageCreateFlags, ShaderStageFlags, PrimitiveTopology, Offset2D, CullModeFlags, FrontFace, PolygonMode, SampleCountFlags, AttachmentReference, CommandBufferUsageFlags, CommandPoolResetFlags, CommandPool, CommandBufferLevel};

use vulkan_rust_test::utility::{constants::*, debug::*, structs::*, context::*, swapchain::*, file_system::*};

const WINDOW_TITLE: &'static str = "10. Hello Triangle!";

use std::fs;
use std::ffi::CString;

const MAX_FRAMES_IN_FLIGHT: usize = 2;
struct FrameSyncContext
{
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
}

impl FrameSyncContext
{
    pub fn destroy_context(&mut self, device: &ash::Device)
    {
        unsafe
        {
            for &semaphore in self.image_available_semaphores.iter()
            {
                device
                    .destroy_semaphore(semaphore, None);
            }
            for &semaphore in self.render_finished_semaphores.iter()
            {
                device
                    .destroy_semaphore(semaphore, None);
            }
            for &fence in self.in_flight_fences.iter()
            {
                device
                    .destroy_fence(fence, None);
            }
        };
    }
}

//We Can Binding One CommandGroup With One Frame, When The Frame Is Finished, So The CommandGroup
struct FrameBoundCommandGroup
{
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    buffer_id: u32,
    level: vk::CommandBufferLevel,
    frame_id: u64,
}

impl FrameBoundCommandGroup
{
    pub fn new(device: &ash::Device, queue_family: &QueueFamilyIndices, level: vk::CommandBufferLevel, frame_id: u64) -> FrameBoundCommandGroup
    {
        let mut result = FrameBoundCommandGroup{
            command_pool: vk::CommandPool::null(),
            command_buffers: vec![],
            buffer_id: 0,
            level,
            frame_id,
        };
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(queue_family.graphics_family.unwrap());

        let fence_create_info = vk::FenceCreateInfo::builder();

        unsafe{
            result.command_pool = device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed To Create Command Pool");
        }

        result
    }

    pub fn allocate_command_buffers(& mut self, device: &ash::Device, count: u32) -> Vec<vk::CommandBuffer>
    {
        let mut result = vec![];
        let available_count = self.command_buffers.len() as u32 - self.buffer_id;
        let using_count = std::cmp::min(available_count, count);
        if using_count > 0 
        {
            for i in 0..using_count
            {
                result.push(self.command_buffers[(i + self.buffer_id) as usize]);
            }
        }

        let remain_count = count - using_count;
        if remain_count > 0 
        {
            let command_buffer_create_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(remain_count)
            .command_pool(self.command_pool)
            .level(self.level);

            let new_commands = unsafe{
                device
                    .allocate_command_buffers(&command_buffer_create_info)
                    .expect("Failed To Create Command Buffer")
            };
            self.command_buffers.extend(new_commands.iter());
            result.extend(new_commands.iter());
        }

        self.buffer_id += count;

        result
    }

    pub fn reset_command_group(& mut self, device: &ash::Device)
    {
        unsafe
        {
            device
            .reset_command_pool(self.command_pool, CommandPoolResetFlags::empty())
            .expect("Failed To Reset Command Pool!");
        }
        self.buffer_id = 0;
    }

    pub fn reset_frame(& mut self, frame_id: u64)
    {
        self.frame_id = frame_id;
    }

    pub fn destroy_group(& mut self, device: &ash::Device)
    {
        unsafe{
            device
            .destroy_command_pool(self.command_pool, None);
        }
        self.command_buffers.clear();
        self.command_pool = CommandPool::null();
        self.buffer_id = 0;
        self.frame_id = 0;
    }
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
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    // command_pool: vk::CommandPool,
    // command_buffers: Vec<vk::CommandBuffer>,

    frame_sync_context: FrameSyncContext,

    current_frame: usize,

    command_groups: Vec<FrameBoundCommandGroup>,
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

        //Swapchain Context
        let swapchain_context = create_swapchain(
            &instance, 
            &logical_device, physical_device, 
            &surface_context, 
            &queue_family_indices);

        //Swapchain Image Views
        let swapchain_image_views = VulkanApp::create_image_views_2d(
            &logical_device,
            swapchain_context.swapchain_format,
            &swapchain_context.swapchain_images);

        //Render Pass
        let render_pass = VulkanApp::create_render_pass(&logical_device, swapchain_context.swapchain_format);

        //Swapchain Framebuffers
        let mut swapchain_framebuffers = vec![];
        for &image_view in swapchain_image_views.iter()
        {
            let frame_buffer = VulkanApp::create_framebuffer(
                &logical_device
                , render_pass
                , &[image_view]
                , &swapchain_context.swapchain_extent);
            swapchain_framebuffers.push(frame_buffer);
        }

        let frame_sync_context = VulkanApp::create_frame_sync_context(&logical_device);

        //Graphics Pipeline
        let (graphics_pipeline, pipeline_layout) = VulkanApp::create_graphics_pipeline(&logical_device, &swapchain_context, render_pass);

        let mut command_groups = vec![];
        for _ in 0..swapchain_image_views.len()
        {
            let command_group = FrameBoundCommandGroup::new(&logical_device, &queue_family_indices, CommandBufferLevel::PRIMARY, 0);
            command_groups.push(command_group);
        }

        // //Command Pool
        // let command_pool = VulkanApp::create_command_pool(&logical_device, &queue_family_indices);

        // //Command Buffers
        // let command_buffers = VulkanApp::create_command_buffers(&logical_device, command_pool, swapchain_image_views.len() as u32);

        // for(i, &command_buffer) in command_buffers.iter().enumerate()
        // {
        //     VulkanApp::record_command_buffer(&logical_device, command_buffer, graphics_pipeline, swapchain_framebuffers[i], render_pass, swapchain_context.swapchain_extent);
        // }

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
            swapchain_framebuffers: swapchain_framebuffers,

            render_pass,
            pipeline_layout,
            graphics_pipeline,

            // command_pool,
            // command_buffers,

            frame_sync_context,

            current_frame: 0,

            command_groups: command_groups,
        }
    }

    fn draw_frame(&mut self)
    {
        // Do Drawing
        let wait_fences = [self.frame_sync_context.in_flight_fences[self.current_frame]];

        let (image_index, _is_sub_optimal) = unsafe{
            self.device
                .wait_for_fences(
                    &wait_fences
                    , true
                    , std::u64::MAX)
                .expect("Waiting Fence Time Out");

            self.swapchain_context.swapchain_loader
                .acquire_next_image(
                    self.swapchain_context.swapchain
                    , std::u64::MAX
                    , self.frame_sync_context.image_available_semaphores[self.current_frame]
                    , vk::Fence::null())
                    .expect("Failed To Aquire Next Frame Image")
        };

        let wait_semaphores = [self.frame_sync_context.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.frame_sync_context.render_finished_semaphores[self.current_frame]];
        //let submit_command_buffers = [self.command_buffers[image_index as usize]];

        let cmd_group = & mut self.command_groups[image_index as usize];
        cmd_group.reset_command_group(&self.device);
        let cmd_buffer = cmd_group.allocate_command_buffers(&self.device, 1);

        VulkanApp::record_command_buffer(&self.device, cmd_buffer[0], self.graphics_pipeline, self.swapchain_framebuffers[image_index as usize], self.render_pass, self.swapchain_context.swapchain_extent);

        let submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&cmd_buffer)
            .signal_semaphores(&signal_semaphores).build()];

        unsafe{
            self.device
                .reset_fences(&wait_fences)
                .expect("Failed To Reset Fence");

            self.device
                .queue_submit(
                    self._graphics_queue
                    , &submit_infos
                    , self.frame_sync_context.in_flight_fences[self.current_frame])
                    .expect("Failed To Execute Queue Submit");
        }

        let swapchains = [self.swapchain_context.swapchain];

        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe{
            self.swapchain_context.swapchain_loader
                .queue_present(self._present_queue, &present_info)
                .expect("Failed To Present Frame");
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
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

    fn create_framebuffer(device: &ash::Device
        , render_pass: vk::RenderPass
        , image_views: &[vk::ImageView]
        , extent: &vk::Extent2D) -> vk::Framebuffer
    {
        let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(image_views)
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        unsafe{
            device
                .create_framebuffer(&frame_buffer_create_info, None)
                .expect("Failed To Create Framebuffer")
        }
    }

    fn create_render_pass(device: &ash::Device, attachment_format: vk::Format) -> vk::RenderPass
    {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(attachment_format)
            .samples(SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let render_pass_attachments = [color_attachment.build()];

        let color_attachment_refs = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];

        let subpasses = [
            vk::SubpassDescription::builder()
                .color_attachments(&color_attachment_refs)
                .build()];

        let subpass_dependencies = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dependency_flags(vk::DependencyFlags::empty())
                .build()];
        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&render_pass_attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        unsafe
        {
            device
                .create_render_pass(&renderpass_create_info, None)
                .expect("Failed To Create Render Pass")
        }
    }

    fn create_graphics_pipeline(device: &ash::Device, swapchain_context: &SwapchainContext, render_pass: vk::RenderPass) -> (vk::Pipeline, vk::PipelineLayout) {
        let shader_src_vert = load_and_compile_shader_src("shaders/testShader.glsl", ShaderType::Vertex);
        let shader_src_frag = load_and_compile_shader_src("shaders/testShader.glsl", ShaderType::Fragment);

        let vertex_module = VulkanApp::create_shader_module(&device, shader_src_vert.as_binary());
        let fragment_module = VulkanApp::create_shader_module(&device, shader_src_frag.as_binary());

        let main_function_name = CString::new("main").unwrap();

        let shader_stage = [vk::PipelineShaderStageCreateInfo::builder()
            .flags(PipelineShaderStageCreateFlags::empty())
            .module(vertex_module)
            .name(&main_function_name)
            .stage(ShaderStageFlags::VERTEX).build(),
            vk::PipelineShaderStageCreateInfo::builder()
            .flags(PipelineShaderStageCreateFlags::empty())
            .module(fragment_module)
            .name(&main_function_name)
            .stage(ShaderStageFlags::FRAGMENT).build()];

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&[])
            .vertex_binding_descriptions(&[]);

        let input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(PrimitiveTopology::TRIANGLE_LIST);

        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain_context.swapchain_extent.width as f32)
            .height(swapchain_context.swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissors = vk::Rect2D::builder()
            .offset(Offset2D{x: 0, y: 0})
            .extent(swapchain_context.swapchain_extent);

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(std::slice::from_ref(&scissors))
            .viewports(std::slice::from_ref(&viewport));

        let rasterizer_state_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::CLOCKWISE)
            .line_width(1.0)
            .polygon_mode(PolygonMode::FILL)
            .rasterizer_discard_enable(false)
            .depth_bias_enable(false)
            .depth_bias_clamp(0.0)
            .depth_bias_constant_factor(0.0)
            .depth_bias_slope_factor(0.0);

        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(0.0)
            .sample_mask(&[])
            .alpha_to_one_enable(false)
            .alpha_to_coverage_enable(false);

        let stencil_state = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .compare_mask(0)
            .write_mask(0)
            .reference(0);

        let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .front(*stencil_state)
            .back(*stencil_state)
            .max_depth_bounds(1.0)
            .min_depth_bounds(0.0);

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .build()];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachment_states)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[])
            .push_constant_ranges(&[]);

        let pipeline_layout = unsafe{
            device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed To Create Pipeline Layout")
        };


        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage)
            .viewport_state(&viewport_state_create_info)
            .vertex_input_state(&vertex_input_state_create_info)
            .input_assembly_state(&input_assembly_state_create_info)
            .rasterization_state(&rasterizer_state_create_info)
            .multisample_state(&multisample_state_create_info)
            .depth_stencil_state(&depth_state_create_info)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let graphics_pipeline = unsafe{
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_create_info),
                    None,
                )
                .expect("Failed To Create Graphics Pipeline")
        };

        unsafe
        {
            device.destroy_shader_module(vertex_module, None);
            device.destroy_shader_module(fragment_module, None);
        }
        (graphics_pipeline[0], pipeline_layout)
    }

    pub fn create_shader_module(device: &ash::Device, binarySource: &[u32]) -> vk::ShaderModule {
        let shader_module_create_info = vk::ShaderModuleCreateInfo::builder()
            .flags(vk::ShaderModuleCreateFlags::empty())
            .code(&binarySource);

        unsafe{
            device
                .create_shader_module(&shader_module_create_info, None)
                .expect("Failed to create Shader Module!")
        }
    }

    fn create_command_pool(device: &ash::Device, queue_family: &QueueFamilyIndices) -> vk::CommandPool
    {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family.graphics_family.unwrap());

        unsafe{
            device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed To Create Command Pool")
        }
    }

    fn create_command_buffers(device: &ash::Device, command_pool: vk::CommandPool, command_buffer_count: u32) -> Vec<vk::CommandBuffer>
    {
        let command_buffer_create_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(command_buffer_count)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        unsafe{
            device
                .allocate_command_buffers(&command_buffer_create_info)
                .expect("Failed To Create Command Buffer")
        }
    }

    fn create_frame_sync_context(device: &ash::Device) -> FrameSyncContext{
        let mut sync_context = FrameSyncContext{
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            in_flight_fences: vec![]
        };

        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();

        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..MAX_FRAMES_IN_FLIGHT{
            unsafe{
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed To Create Semaphore");

                let render_finish_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed To Create Semaphore");

                let in_flight_fence = device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed To Create Fence");

                sync_context.image_available_semaphores.push(image_available_semaphore);
                sync_context.render_finished_semaphores.push(render_finish_semaphore);
                sync_context.in_flight_fences.push(in_flight_fence);
            }
        }

        sync_context
    }

    fn record_command_buffer(device: &ash::Device, command_buffer: vk::CommandBuffer
        , graphics_pipeline: vk::Pipeline
        , framebuffer: vk::Framebuffer
        , render_pass: vk::RenderPass
        , extent: vk::Extent2D)
    {
        let clear_values = [
            vk::ClearValue{
                color: vk::ClearColorValue{
                    float32: [0.0, 0.0, 0.0, 1.0]
                }
            }
        ];

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(CommandBufferUsageFlags::SIMULTANEOUS_USE);

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .clear_values(&clear_values)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D{
                offset: vk::Offset2D{x: 0, y: 0},
                extent: extent
            });

        unsafe{
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Begin Command Buffer Failed!");

            device
                .cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);

            device
                .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);

            device
                .cmd_draw(command_buffer, 3, 1, 0, 0);

            device
                .cmd_end_render_pass(command_buffer);

            device
                .end_command_buffer(command_buffer)
                .expect("Failed To End Command Buffer");
        }
    }

}

impl Drop for VulkanApp
{
    fn drop(&mut self)
    {
        unsafe
        {
            //wait idle before destruction
            self.device.device_wait_idle()
                .expect("Device Wait Idle Failed!");

            //destroy sync context
            self.frame_sync_context.destroy_context(&self.device);

            //Destroy Command Group(Command Buffer Shall Be Released Together With Pools)
            for cmd_group in self.command_groups.iter_mut()
            {
                cmd_group.destroy_group(&self.device);
            }
            //Pipeline
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            //Pipeline Layout
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            //Framebuffers
            for &framebuffer in self.swapchain_framebuffers.iter()
            {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            //RenderPass
            self.device.destroy_render_pass(self.render_pass, None);
            //Image Views
            for &image_view in self.swapchain_image_views.iter()
            {
                self.device.destroy_image_view(image_view, None);
            }
            //Swapchain
            self.swapchain_context.swapchain_loader.destroy_swapchain(self.swapchain_context.swapchain, None);
            //Device
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
    let contents = fs::read_to_string("testStr.txt");
    println!("Text: {0}", contents.unwrap());

    let event_loop = EventLoop::new();
    let window = VulkanApp::init_window(&event_loop);

    let vulkan_app = VulkanApp::new(&window);
    vulkan_app.main_loop(event_loop, window);
}