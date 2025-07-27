use ash::{vk};

use super::{utils::UniformBufferObject, VulkanApp};

pub fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
    let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .build();

    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(std::slice::from_ref(&ubo_layout_binding));

    unsafe { device.create_descriptor_set_layout(&layout_info, None).unwrap() }
}

pub fn create_descriptor_pool(
    device: &ash::Device,
    num_images: usize,
    descriptor_set_layout: vk::DescriptorSetLayout,
) -> (vk::DescriptorPool, Vec<vk::DescriptorSet>) {
    let pool_size = vk::DescriptorPoolSize::builder()
        .ty(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(100)
        .build();

    let pool_info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(std::slice::from_ref(&pool_size))
        .max_sets(100);

    let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None).unwrap() };

    let layouts = vec![descriptor_set_layout; num_images];
    let allocate_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);

    let descriptor_sets = unsafe { device.allocate_descriptor_sets(&allocate_info).unwrap() };

    (descriptor_pool, descriptor_sets)
}

pub fn create_descriptor_sets(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    uniform_buffers: &[vk::Buffer],
    num_images: usize,
) -> Vec<vk::DescriptorSet> {
    let layouts = vec![descriptor_set_layout; num_images];
    let alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);

    let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info).unwrap() };

    for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffers[i])
            .offset(0)
            .range(std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize)
            .build();

        let descriptor_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info))
            .build();

        unsafe { device.update_descriptor_sets(std::slice::from_ref(&descriptor_write), &[]) };
    }

    descriptor_sets
}
