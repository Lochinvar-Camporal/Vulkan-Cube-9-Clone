use ash::{vk};

use super::{buffers, VulkanApp};

pub fn create_depth_resources(
    instance: &ash::Instance,
    device: &ash::Device,
    pdevice: vk::PhysicalDevice,
    extent: vk::Extent2D,
) -> (vk::Image, vk::DeviceMemory, vk::ImageView) {
    let depth_format = find_depth_format(instance, pdevice);
    let (depth_image, depth_image_memory) = create_image(
        instance,
        device,
        pdevice,
        extent.width,
        extent.height,
        depth_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );
    let depth_image_view = create_image_view(device, depth_image, depth_format, vk::ImageAspectFlags::DEPTH);

    (depth_image, depth_image_memory, depth_image_view)
}

pub fn find_depth_format(instance: &ash::Instance, pdevice: vk::PhysicalDevice) -> vk::Format {
    find_supported_format(
        instance,
        pdevice,
        &[vk::Format::D32_SFLOAT, vk::Format::D32_SFLOAT_S8_UINT, vk::Format::D24_UNORM_S8_UINT],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}

fn find_supported_format(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    for &format in candidates {
        let props = unsafe { instance.get_physical_device_format_properties(pdevice, format) };

        if tiling == vk::ImageTiling::LINEAR && props.linear_tiling_features.contains(features) {
            return format;
        } else if tiling == vk::ImageTiling::OPTIMAL && props.optimal_tiling_features.contains(features) {
            return format;
        }
    }
    panic!("Failed to find supported format!");
}

pub fn create_image(
    instance: &ash::Instance,
    device: &ash::Device,
    pdevice: vk::PhysicalDevice,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> (vk::Image, vk::DeviceMemory) {
    let image_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D { width, height, depth: 1 })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::TYPE_1);

    let image = unsafe { device.create_image(&image_info, None).unwrap() };

    let mem_requirements = unsafe { device.get_image_memory_requirements(image) };
    let mem_type_index = buffers::find_memory_type(instance, pdevice, mem_requirements.memory_type_bits, properties);

    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(mem_requirements.size)
        .memory_type_index(mem_type_index);

    let image_memory = unsafe { device.allocate_memory(&alloc_info, None).unwrap() };
    unsafe {
        device.bind_image_memory(image, image_memory, 0).unwrap();
    }

    (image, image_memory)
}

pub fn create_image_view(
    device: &ash::Device,
    image: vk::Image,
    format: vk::Format,
    aspect_flags: vk::ImageAspectFlags,
) -> vk::ImageView {
    let view_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: aspect_flags,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });

    unsafe { device.create_image_view(&view_info, None).unwrap() }
}
