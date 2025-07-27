use ash::{vk, Entry};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use super::utils::{QueueFamilyIndices, UniformBufferObject};
use super::vertex::{Vertex, INDICES, VERTICES, generate_wireframe_vertices};
use super::{buffers, commands, descriptors, images, instance, pipeline, swapchain};

pub struct VulkanApp {
    pub(super) entry: Entry,
    pub(super) instance: ash::Instance,
    pub(super) debug_utils_loader: ash::extensions::ext::DebugUtils,
    pub(super) debug_messenger: vk::DebugUtilsMessengerEXT,
    pub(super) surface: vk::SurfaceKHR,
    pub(super) surface_loader: ash::extensions::khr::Surface,
    pub(super) physical_device: vk::PhysicalDevice,
    pub(super) device: ash::Device,
    pub(super) graphics_queue: vk::Queue,
    pub(super) present_queue: vk::Queue,
    pub(super) swapchain_loader: ash::extensions::khr::Swapchain,
    pub(super) swapchain: vk::SwapchainKHR,
    pub(super) swapchain_images: Vec<vk::Image>,
    pub(super) swapchain_format: vk::Format,
    pub(super) swapchain_extent: vk::Extent2D,
    pub(super) swapchain_image_views: Vec<vk::ImageView>,
    pub(super) render_pass: vk::RenderPass,
    pub(super) pipeline_layout: vk::PipelineLayout,
    pub(super) graphics_pipeline: vk::Pipeline,
    pub(super) wireframe_pipeline: vk::Pipeline,
    pub(super) framebuffers: Vec<vk::Framebuffer>,
    pub(super) command_pool: vk::CommandPool,
    pub(super) command_buffers: Vec<vk::CommandBuffer>,
    pub(super) image_available_semaphore: vk::Semaphore,
    pub(super) render_finished_semaphore: vk::Semaphore,
    pub(super) in_flight_fence: vk::Fence,
    pub framebuffer_resized: bool,
    pub(super) queue_family_indices: QueueFamilyIndices,
    pub(super) vertex_buffer: vk::Buffer,
    pub(super) vertex_buffer_memory: vk::DeviceMemory,
    pub(super) wireframe_vertex_buffer: vk::Buffer,
    pub(super) wireframe_vertex_buffer_memory: vk::DeviceMemory,
    pub(super) wireframe_vertex_count: u32,
    pub(super) index_buffer: vk::Buffer,
    pub(super) index_buffer_memory: vk::DeviceMemory,
    pub(super) uniform_buffers: Vec<vk::Buffer>,
    pub(super) uniform_buffers_memory: Vec<vk::DeviceMemory>,
    pub(super) descriptor_set_layout: vk::DescriptorSetLayout,
    pub(super) descriptor_pool: vk::DescriptorPool,
    pub(super) descriptor_sets: Vec<vk::DescriptorSet>,
    pub(super) depth_image: vk::Image,
    pub(super) depth_image_memory: vk::DeviceMemory,
    pub(super) depth_image_view: vk::ImageView,
}

impl VulkanApp {
    pub fn new(window: &winit::window::Window) -> Self {
        let entry = unsafe { Entry::load().unwrap() };
        let instance = instance::create_instance(&entry, window);
        let (debug_utils_loader, debug_messenger) = instance::setup_debug_messenger(&entry, &instance);
        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
            .unwrap()
        };
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
        let (physical_device, queue_family_indices) =
            instance::pick_physical_device(&instance, &surface_loader, surface);
        let (device, graphics_queue, present_queue) =
            instance::create_logical_device(&instance, physical_device, &queue_family_indices);

        let (vertex_buffer, vertex_buffer_memory) = buffers::create_vertex_buffer(
            &instance,
            &device,
            physical_device,
            &queue_family_indices,
            &VERTICES,
        );
        let wire_vertices = generate_wireframe_vertices(24);
        let wireframe_vertex_count = wire_vertices.len() as u32;
        let (wireframe_vertex_buffer, wireframe_vertex_buffer_memory) =
            buffers::create_vertex_buffer(
                &instance,
                &device,
                physical_device,
                &queue_family_indices,
                &wire_vertices,
            );
        let (index_buffer, index_buffer_memory) = buffers::create_index_buffer(
            &instance,
            &device,
            physical_device,
            &queue_family_indices,
            &INDICES,
        );

        let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &device);
        let (swapchain, swapchain_format, swapchain_extent) = swapchain::create_swapchain(
            &instance,
            &device,
            physical_device,
            &surface_loader,
            surface,
            &queue_family_indices,
            &swapchain_loader,
            window,
        );
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        let swapchain_image_views =
            swapchain::create_image_views(&device, &swapchain_images, swapchain_format);
        let depth_format = images::find_depth_format(&instance, physical_device);
        let descriptor_set_layout = descriptors::create_descriptor_set_layout(&device);
        let render_pass = pipeline::create_render_pass(&device, swapchain_format, depth_format);
        let (graphics_pipeline, pipeline_layout) = pipeline::create_graphics_pipeline(
            &device,
            render_pass,
            swapchain_extent,
            descriptor_set_layout,
        );
        let wireframe_pipeline = pipeline::create_wireframe_pipeline(
            &device,
            render_pass,
            swapchain_extent,
            pipeline_layout,
        );
        let (depth_image, depth_image_memory, depth_image_view) = images::create_depth_resources(
            &instance,
            &device,
            physical_device,
            swapchain_extent,
        );
        let framebuffers = pipeline::create_framebuffers(
            &device,
            &swapchain_image_views,
            depth_image_view,
            render_pass,
            swapchain_extent,
        );
        let command_pool = commands::create_command_pool(&device, &queue_family_indices);
        let command_buffers =
            commands::create_command_buffers(&device, command_pool, framebuffers.len());
        let (image_available_semaphore, render_finished_semaphore, in_flight_fence) =
            commands::create_sync_objects(&device);

        let (uniform_buffers, uniform_buffers_memory) = buffers::create_uniform_buffers(
            &instance,
            &device,
            physical_device,
            swapchain_images.len(),
        );
        let (descriptor_pool, descriptor_sets) = descriptors::create_descriptor_pool(
            &device,
            swapchain_images.len(),
            descriptor_set_layout,
        );
        let descriptor_sets = descriptors::create_descriptor_sets(
            &device,
            descriptor_pool,
            descriptor_set_layout,
            &uniform_buffers,
            swapchain_images.len(),
        );

        Self {
            entry,
            instance,
            debug_utils_loader,
            debug_messenger,
            surface,
            surface_loader,
            physical_device,
            device,
            graphics_queue,
            present_queue,
            swapchain_loader,
            swapchain,
            swapchain_images,
            swapchain_format,
            swapchain_extent,
            swapchain_image_views,
            render_pass,
            pipeline_layout,
            graphics_pipeline,
            wireframe_pipeline,
            framebuffers,
            command_pool,
            command_buffers,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
            framebuffer_resized: false,
            queue_family_indices,
            vertex_buffer,
            vertex_buffer_memory,
            wireframe_vertex_buffer,
            wireframe_vertex_buffer_memory,
            wireframe_vertex_count,
            index_buffer,
            index_buffer_memory,
            uniform_buffers,
            uniform_buffers_memory,
            descriptor_set_layout,
            descriptor_pool,
            descriptor_sets,
            depth_image,
            depth_image_memory,
            depth_image_view,
        }
    }

    pub fn draw_frame(&mut self, window: &winit::window::Window, camera: &crate::camera::Camera) {
        unsafe {
            self.device
                .wait_for_fences(std::slice::from_ref(&self.in_flight_fence), true, u64::MAX)
                .unwrap();

            let result = self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
            );

            let image_index = match result {
                Ok((image_index, is_suboptimal)) => {
                    if is_suboptimal {
                        self.framebuffer_resized = true;
                    }
                    image_index
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.recreate_swapchain(window);
                    return;
                }
                Err(error) => panic!("Error acquiring swapchain image: {}", error),
            };

            self.update_uniform_buffer(image_index as usize, camera);

            self.device
                .reset_fences(std::slice::from_ref(&self.in_flight_fence))
                .unwrap();

            self.device
                .reset_command_buffer(
                    self.command_buffers[image_index as usize],
                    vk::CommandBufferResetFlags::empty(),
                )
                .unwrap();
            self.record_command_buffer(
                self.command_buffers[image_index as usize],
                image_index as usize,
            );

            let wait_semaphores = [self.image_available_semaphore];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores = [self.render_finished_semaphore];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(std::slice::from_ref(
                    &self.command_buffers[image_index as usize],
                ))
                .signal_semaphores(&signal_semaphores);

            self.device
                .queue_submit(
                    self.graphics_queue,
                    std::slice::from_ref(&submit_info),
                    self.in_flight_fence,
                )
                .unwrap();

            let swapchains = [self.swapchain];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(std::slice::from_ref(&image_index));

            let result = self
                .swapchain_loader
                .queue_present(self.present_queue, &present_info);

            let mut recreate_swapchain = false;
            match result {
                Ok(is_suboptimal) => {
                    if is_suboptimal {
                        recreate_swapchain = true;
                    }
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                    recreate_swapchain = true;
                }
                Err(error) => panic!("Failed to present swapchain image: {}", error),
            }

            if self.framebuffer_resized || recreate_swapchain {
                self.framebuffer_resized = false;
                self.recreate_swapchain(window);
            }
        }
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.cleanup_swapchain();
            self.device.destroy_buffer(self.index_buffer, None);
            self.device.free_memory(self.index_buffer_memory, None);
            self.device.destroy_buffer(self.wireframe_vertex_buffer, None);
            self.device.free_memory(self.wireframe_vertex_buffer_memory, None);
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
            self.device
                .destroy_semaphore(self.image_available_semaphore, None);
            self.device
                .destroy_semaphore(self.render_finished_semaphore, None);
            self.device.destroy_fence(self.in_flight_fence, None);
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);
            self.device.free_memory(self.depth_image_memory, None);
            self.device.destroy_pipeline(self.wireframe_pipeline, None);
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            for i in 0..self.uniform_buffers.len() {
                self.device.destroy_buffer(self.uniform_buffers[i], None);
                self.device
                    .free_memory(self.uniform_buffers_memory[i], None);
            }
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}
