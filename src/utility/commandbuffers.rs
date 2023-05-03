use ash::vk;
use ash::vk::{CommandPoolResetFlags, CommandPool};
use crate::utility::{structs};

//We Can Binding One CommandGroup With One Frame, When The Frame Is Finished, So The CommandGroup
#[derive(Debug, Clone)]
pub struct FrameBoundCommandGroup
{
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    buffer_id: u32,
    level: vk::CommandBufferLevel,
    frame_id: u64,
}

impl FrameBoundCommandGroup
{
    pub fn new(device: &ash::Device, queue_family: &structs::QueueFamilyIndices, level: vk::CommandBufferLevel, frame_id: u64) -> FrameBoundCommandGroup
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


pub struct OneTimeSubmitCommandGroup
{
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    buffer_id: u32,
    level: vk::CommandBufferLevel,
    fence: vk::Fence,
}

impl OneTimeSubmitCommandGroup
{
    pub fn new(device: &ash::Device, queue_family: &structs::QueueFamilyIndices, level: vk::CommandBufferLevel) -> OneTimeSubmitCommandGroup
    {
        let mut result = OneTimeSubmitCommandGroup{
            command_pool: vk::CommandPool::null(),
            command_buffers: vec![],
            buffer_id: 0,
            level,
            fence: vk::Fence::null(),
        };
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(queue_family.graphics_family.unwrap());

        let fence_create_info = vk::FenceCreateInfo::builder();

        unsafe{
            result.command_pool = device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed To Create Command Pool");

            result.fence = device
                .create_fence(&fence_create_info, None)
                .expect("Failed To Create Fence");
        }

        result
    }

    pub fn destroy_group(& mut self, device: &ash::Device)
    {
        unsafe{
            device
                .destroy_command_pool(self.command_pool, None);
            device
                .destroy_fence(self.fence, None);
        }
        self.command_buffers.clear();
        self.command_pool = CommandPool::null();
        self.fence = vk::Fence::null();
        self.buffer_id = 0;
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

    pub fn submit_and_wait(&mut self, device: &ash::Device, queue: vk::Queue)
    {
        let submit_infos = [vk::SubmitInfo::builder()
        .command_buffers(&self.command_buffers[..self.buffer_id as usize]).build()];
        unsafe
        {
            device
                .queue_submit(queue, &submit_infos, self.fence)
                .expect("Failed To Submit Fences");

            device
                .wait_for_fences(
                    &[self.fence]
                    , true
                    , std::u64::MAX)
                .expect("Waiting One Time Submit Fence Time Out");

            device
                .reset_fences(&[self.fence])
                .expect("Failed To Reset One Time Submit Fence");

            device
                .reset_command_pool(self.command_pool, CommandPoolResetFlags::empty())
                .expect("Failed To Reset Command Pool!");
        }
        self.buffer_id = 0;
    }

}