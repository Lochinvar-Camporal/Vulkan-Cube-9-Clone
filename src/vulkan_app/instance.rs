use ash::{vk, Entry};
use raw_window_handle::HasRawDisplayHandle;
use std::ffi::{CStr, CString};

use super::utils::{vulkan_debug_callback, QueueFamilyIndices, SwapchainSupportDetails};

pub fn create_instance(entry: &Entry, window: &winit::window::Window) -> ash::Instance {
    let app_name = CString::new("Vulkan Triangle").unwrap();
    let engine_name = CString::new("No Engine").unwrap();
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_api_version(0, 1, 0, 0))
        .engine_name(&engine_name)
        .engine_version(vk::make_api_version(0, 1, 0, 0))
        .api_version(vk::API_VERSION_1_0);

    let mut extension_names =
        ash_window::enumerate_required_extensions(window.raw_display_handle())
            .unwrap()
            .to_vec();
    extension_names.push(ash::extensions::ext::DebugUtils::name().as_ptr());

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names);

    unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Failed to create instance")
    }
}

pub fn setup_debug_messenger(
    entry: &Entry,
    instance: &ash::Instance,
) -> (ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT) {
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback));

    let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);
    let debug_messenger = unsafe {
        debug_utils_loader
            .create_debug_utils_messenger(&debug_info, None)
            .unwrap()
    };

    (debug_utils_loader, debug_messenger)
}

pub fn pick_physical_device(
    instance: &ash::Instance,
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
) -> (vk::PhysicalDevice, QueueFamilyIndices) {
    let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };
    let physical_device = physical_devices
        .into_iter()
        .find(|pdevice| is_device_suitable(instance, surface_loader, surface, *pdevice))
        .expect("Failed to find a suitable GPU!");

    let indices = find_queue_families(instance, surface_loader, surface, physical_device);
    (physical_device, indices)
}

fn is_device_suitable(
    instance: &ash::Instance,
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    pdevice: vk::PhysicalDevice,
) -> bool {
    let indices = find_queue_families(instance, surface_loader, surface, pdevice);
    let extensions_supported = check_device_extension_support(instance, pdevice);

    let mut swapchain_adequate = false;
    if extensions_supported {
        let swapchain_support = super::swapchain::query_swapchain_support(surface_loader, pdevice, surface);
        swapchain_adequate = !swapchain_support.formats.is_empty()
            && !swapchain_support.present_modes.is_empty();
    }

    indices.is_complete() && extensions_supported && swapchain_adequate
}

fn check_device_extension_support(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
) -> bool {
    let required_extensions = [ash::extensions::khr::Swapchain::name()];
    let available_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(pdevice)
            .unwrap()
    };

    for required in required_extensions.iter() {
        let found = available_extensions.iter().any(|ext| {
            let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
            required == &name
        });

        if !found {
            return false;
        }
    }

    true
}

pub fn find_queue_families(
    instance: &ash::Instance,
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    pdevice: vk::PhysicalDevice,
) -> QueueFamilyIndices {
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(pdevice) };
    let mut indices = QueueFamilyIndices::new();

    for (i, queue_family) in queue_families.iter().enumerate() {
        if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            indices.graphics_family = Some(i as u32);
        }

        let present_support = unsafe {
            surface_loader
                .get_physical_device_surface_support(pdevice, i as u32, surface)
                .unwrap()
        };
        if present_support {
            indices.present_family = Some(i as u32);
        }

        if indices.is_complete() {
            break;
        }
    }

    indices
}

pub fn create_logical_device(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    indices: &QueueFamilyIndices,
) -> (ash::Device, vk::Queue, vk::Queue) {
    let mut unique_queue_families = std::collections::HashSet::new();
    unique_queue_families.insert(indices.graphics_family.unwrap());
    unique_queue_families.insert(indices.present_family.unwrap());

    let queue_priorities = [1.0];
    let mut queue_create_infos = vec![];
    for queue_family in unique_queue_families {
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family)
            .queue_priorities(&queue_priorities)
            .build();
        queue_create_infos.push(queue_create_info);
    }

    let physical_device_features = vk::PhysicalDeviceFeatures::builder();
    let required_extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];

    let create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_features(&physical_device_features)
        .enabled_extension_names(&required_extensions);

    let device = unsafe { instance.create_device(pdevice, &create_info, None).unwrap() };

    let graphics_queue = unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };
    let present_queue = unsafe { device.get_device_queue(indices.present_family.unwrap(), 0) };

    (device, graphics_queue, present_queue)
}
