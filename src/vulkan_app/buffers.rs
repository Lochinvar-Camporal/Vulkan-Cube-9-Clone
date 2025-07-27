use ash::vk;

use cgmath::{Matrix4, SquareMatrix};


use super::{utils::{QueueFamilyIndices, UniformBufferObject}, vertex::{Vertex, INDICES}, VulkanApp};

pub fn create_index_buffer(
    instance: &ash::Instance,
    device: &ash::Device,
    pdevice: vk::PhysicalDevice,
    _indices: &QueueFamilyIndices,
    data: &[u16],
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_size = (std::mem::size_of::<u16>() * INDICES.len()) as vk::DeviceSize;
    let (buffer, buffer_memory) = create_buffer(
        instance,
        device,
        pdevice,
        buffer_size,
        vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    unsafe {
        let data_ptr = device
            .map_memory(buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
            .unwrap();
        let mut align = ash::util::Align::new(data_ptr, std::mem::align_of::<u16>() as _, buffer_size);
        align.copy_from_slice(data);
        device.unmap_memory(buffer_memory);
    }

    (buffer, buffer_memory)
}

pub fn create_vertex_buffer(
    instance: &ash::Instance,
    device: &ash::Device,
    pdevice: vk::PhysicalDevice,
    _indices: &QueueFamilyIndices,
    data: &[Vertex],
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_size = (std::mem::size_of::<Vertex>() * data.len()) as vk::DeviceSize;
    let (buffer, buffer_memory) = create_buffer(
        instance,
        device,
        pdevice,
        buffer_size,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    unsafe {
        let data_ptr = device
            .map_memory(buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
            .unwrap();
        let mut align = ash::util::Align::new(data_ptr, std::mem::align_of::<Vertex>() as _, buffer_size);
        align.copy_from_slice(data);
        device.unmap_memory(buffer_memory);
    }

    (buffer, buffer_memory)
}

pub fn create_buffer(
    instance: &ash::Instance,
    device: &ash::Device,
    pdevice: vk::PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_info, None).unwrap() };
    let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let mem_type_index = find_memory_type(
        instance,
        pdevice,
        mem_requirements.memory_type_bits,
        properties,
    );

    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(mem_requirements.size)
        .memory_type_index(mem_type_index);

    let buffer_memory = unsafe { device.allocate_memory(&alloc_info, None).unwrap() };
    unsafe {
        device.bind_buffer_memory(buffer, buffer_memory, 0).unwrap();
    }

    (buffer, buffer_memory)
}

pub fn find_memory_type(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> u32 {
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(pdevice) };
    for i in 0..mem_properties.memory_type_count {
        if (type_filter & (1 << i)) != 0
            && (mem_properties.memory_types[i as usize].property_flags.contains(properties))
        {
            return i;
        }
    }
    panic!("Failed to find suitable memory type!");
}

pub fn create_uniform_buffers(
    instance: &ash::Instance,
    device: &ash::Device,
    pdevice: vk::PhysicalDevice,
    num_images: usize,
) -> (Vec<vk::Buffer>, Vec<vk::DeviceMemory>) {
    let buffer_size = std::mem::size_of::<UniformBufferObject>();
    let mut uniform_buffers = Vec::with_capacity(num_images);
    let mut uniform_buffers_memory = Vec::with_capacity(num_images);

    for _ in 0..num_images {
        let (buffer, memory) = create_buffer(
            instance,
            device,
            pdevice,
            buffer_size as vk::DeviceSize,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        uniform_buffers.push(buffer);
        uniform_buffers_memory.push(memory);
    }

    (uniform_buffers, uniform_buffers_memory)
}

impl VulkanApp {
    pub fn update_uniform_buffer(&self, current_image: usize, camera: &crate::camera::Camera) {
        let model = Matrix4::identity();
        let view = camera.view_matrix();
        let mut proj = cgmath::perspective(
            cgmath::Deg(45.0),
            self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32,
            0.1,
            100.0,
        );
        proj[1][1] *= -1.0;

        let ubo = UniformBufferObject { model, view, proj };

        unsafe {
            let data_ptr = self
                .device
                .map_memory(
                    self.uniform_buffers_memory[current_image],
                    0,
                    std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap();
            let mut align = ash::util::Align::new(
                data_ptr,
                std::mem::align_of::<UniformBufferObject>() as _,
                std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize,
            );
            align.copy_from_slice(&[ubo]);
            self.device
                .unmap_memory(self.uniform_buffers_memory[current_image]);
        }
    }
}
