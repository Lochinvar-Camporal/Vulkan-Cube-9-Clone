use ash::{vk};
use winit::window::Window;

use super::{utils::{QueueFamilyIndices, SwapchainSupportDetails}, images, pipeline, descriptors, buffers, VulkanApp};

pub fn create_swapchain(
    instance: &ash::Instance,
    device: &ash::Device,
    pdevice: vk::PhysicalDevice,
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    indices: &QueueFamilyIndices,
    swapchain_loader: &ash::extensions::khr::Swapchain,
    window: &Window,
) -> (vk::SwapchainKHR, vk::Format, vk::Extent2D) {
    let swapchain_support = query_swapchain_support(surface_loader, pdevice, surface);
    let surface_format = choose_swap_surface_format(&swapchain_support.formats);
    let present_mode = choose_swap_present_mode(&swapchain_support.present_modes);
    let extent = choose_swap_extent(&swapchain_support.capabilities, window);

    let mut image_count = swapchain_support.capabilities.min_image_count + 1;
    if swapchain_support.capabilities.max_image_count > 0
        && image_count > swapchain_support.capabilities.max_image_count
    {
        image_count = swapchain_support.capabilities.max_image_count;
    }

    let mut create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

    let queue_family_indices = [indices.graphics_family.unwrap(), indices.present_family.unwrap()];

    if indices.graphics_family != indices.present_family {
        create_info = create_info
            .image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(&queue_family_indices);
    } else {
        create_info = create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
    }

    let create_info = create_info
        .pre_transform(swapchain_support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true);

    let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None).unwrap() };

    (swapchain, surface_format.format, extent)
}

pub fn query_swapchain_support(
    surface_loader: &ash::extensions::khr::Surface,
    pdevice: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> SwapchainSupportDetails {
    let capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(pdevice, surface)
            .unwrap()
    };
    let formats = unsafe {
        surface_loader
            .get_physical_device_surface_formats(pdevice, surface)
            .unwrap()
    };
    let present_modes = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(pdevice, surface)
            .unwrap()
    };

    SwapchainSupportDetails {
        capabilities,
        formats,
        present_modes,
    }
}

fn choose_swap_surface_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    *available_formats
        .iter()
        .find(|format| {
            format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or(&available_formats[0])
}

fn choose_swap_present_mode(available_present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    *available_present_modes
        .iter()
        .find(|mode| **mode == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(&vk::PresentModeKHR::FIFO)
}

fn choose_swap_extent(capabilities: &vk::SurfaceCapabilitiesKHR, window: &Window) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        let inner_size = window.inner_size();
        vk::Extent2D {
            width: inner_size.width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: inner_size.height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
}

pub fn create_image_views(
    device: &ash::Device,
    images: &[vk::Image],
    format: vk::Format,
) -> Vec<vk::ImageView> {
    images
        .iter()
        .map(|&image| {
            let create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            unsafe { device.create_image_view(&create_info, None).unwrap() }
        })
        .collect()
}

impl VulkanApp {
    pub fn cleanup_swapchain(&mut self) {
        unsafe {
            for i in 0..self.uniform_buffers.len() {
                self.device.destroy_buffer(self.uniform_buffers[i], None);
                self.device
                    .free_memory(self.uniform_buffers_memory[i], None);
            }
            for framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.destroy_pipeline(self.wireframe_pipeline, None);
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            for image_view in self.swapchain_image_views.iter() {
                self.device.destroy_image_view(*image_view, None);
            }
            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);
            self.device.free_memory(self.depth_image_memory, None);
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }

    pub fn recreate_swapchain(&mut self, window: &Window) {
        unsafe {
            self.device.device_wait_idle().unwrap();
        }
        self.cleanup_swapchain();

        let depth_format = images::find_depth_format(&self.instance, self.physical_device);
        self.render_pass = pipeline::create_render_pass(&self.device, self.swapchain_format, depth_format);

        let (swapchain, swapchain_format, swapchain_extent) = create_swapchain(
            &self.instance,
            &self.device,
            self.physical_device,
            &self.surface_loader,
            self.surface,
            &self.queue_family_indices,
            &self.swapchain_loader,
            window,
        );
        self.swapchain = swapchain;
        self.swapchain_images = unsafe { self.swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        self.swapchain_format = swapchain_format;
        self.swapchain_extent = swapchain_extent;
        self.swapchain_image_views =
            create_image_views(&self.device, &self.swapchain_images, self.swapchain_format);
        let depth_format = images::find_depth_format(&self.instance, self.physical_device);
        self.render_pass = pipeline::create_render_pass(&self.device, self.swapchain_format, depth_format);
        let (graphics_pipeline, pipeline_layout) = pipeline::create_graphics_pipeline(
            &self.device,
            self.render_pass,
            self.swapchain_extent,
            self.descriptor_set_layout,
        );
        self.graphics_pipeline = graphics_pipeline;
        self.pipeline_layout = pipeline_layout;
        self.wireframe_pipeline = pipeline::create_wireframe_pipeline(
            &self.device,
            self.render_pass,
            self.swapchain_extent,
            self.pipeline_layout,
        );
        let (depth_image, depth_image_memory, depth_image_view) = images::create_depth_resources(
            &self.instance,
            &self.device,
            self.physical_device,
            self.swapchain_extent,
        );
        self.depth_image = depth_image;
        self.depth_image_memory = depth_image_memory;
        self.depth_image_view = depth_image_view;
        self.framebuffers = pipeline::create_framebuffers(
            &self.device,
            &self.swapchain_image_views,
            self.depth_image_view,
            self.render_pass,
            self.swapchain_extent,
        );
        let (uniform_buffers, uniform_buffers_memory) = buffers::create_uniform_buffers(
            &self.instance,
            &self.device,
            self.physical_device,
            self.swapchain_images.len(),
        );
        self.uniform_buffers = uniform_buffers;
        self.uniform_buffers_memory = uniform_buffers_memory;
        let (descriptor_pool, descriptor_sets) = descriptors::create_descriptor_pool(
            &self.device,
            self.swapchain_images.len(),
            self.descriptor_set_layout,
        );
        self.descriptor_pool = descriptor_pool;
        self.descriptor_sets = descriptor_sets;
        self.descriptor_sets = descriptors::create_descriptor_sets(
            &self.device,
            self.descriptor_pool,
            self.descriptor_set_layout,
            &self.uniform_buffers,
            self.swapchain_images.len(),
        );
    }
}
