pub const WIDTH: u32 = 800;
pub const HEIGHT: u32 = 600;

pub use app::VulkanApp;

mod app;
mod utils;
mod vertex;

mod instance;
mod swapchain;
mod pipeline;
mod buffers;
mod images;
mod commands;
mod descriptors;
