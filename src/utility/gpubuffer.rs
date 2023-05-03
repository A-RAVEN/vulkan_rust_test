
use ash::vk;
use gpu_allocator::vulkan;

//use crate::utility:
pub struct GPUBuffer
{
    pub buffer: vk::Buffer,
    pub allocation: vulkan::Allocation,
}

impl GPUBuffer
{
    pub fn create_gpu_buffer(
        device:& ash::Device
        ,allocator: &mut vulkan::Allocator
        ,buffer_size: u64
        ,buffer_usage: vk::BufferUsageFlags
        ,memory_location: gpu_allocator::MemoryLocation) -> GPUBuffer
    {
        let gpu_buffer_create_info = vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(buffer_usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&[]);

        let gpu_buffer = unsafe{
            device
                .create_buffer(&gpu_buffer_create_info, None)
                .expect("Failed To Create Vertex Buffer!")
        };

        let allocation_requirement = unsafe{
            device
                .get_buffer_memory_requirements(gpu_buffer)
        };

        let allocation = allocator.allocate(&vulkan::AllocationCreateDesc{
            name: "Custom Vertex Buffer",
            requirements: allocation_requirement,
            location: memory_location,
            linear: true,
            allocation_scheme: vulkan::AllocationScheme::GpuAllocatorManaged,
        }).expect("Failed To Create Buffer Memory Allocation!");

        unsafe{
            device
                .bind_buffer_memory(gpu_buffer, allocation.memory(), allocation.offset())
                .expect("Failed To Bind Memory To Vertex Buffer!")
        };

        GPUBuffer
        {
            buffer: gpu_buffer,
            allocation: allocation,
        }
    }

    pub fn destroy_gpu_buffer(
        gpu_buffer: &mut GPUBuffer
        , device:& ash::Device
        , allocator: &mut vulkan::Allocator)
    {
        unsafe{
            device.destroy_buffer(gpu_buffer.buffer, None);
        }
        let allocation = std::mem::take(&mut gpu_buffer.allocation);
        allocator
            .free(allocation)
            .expect("Failed To Destroy GPU Buffer!");
    }

    pub unsafe fn cmd_copy_buffer(
        device: &ash::Device
        ,command_buffer: vk::CommandBuffer
        ,src_buffer: &GPUBuffer
        ,dst_buffer: &GPUBuffer 
    )
    {
        let copy_regions = [vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: src_buffer.allocation.size(),
        }];

        device
            .cmd_copy_buffer(
                command_buffer
                , src_buffer.buffer
                , dst_buffer.buffer
                , &copy_regions);

    }
}