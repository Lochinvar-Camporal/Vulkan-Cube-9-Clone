mod vulkan_app;
mod camera;

use vulkan_app::{VulkanApp, HEIGHT, WIDTH};
use camera::{Camera, CameraMovement};
use winit::event::{Event, WindowEvent, DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Triangle")
        .with_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)
        .unwrap();

    let mut app = VulkanApp::new(&window);
    let mut camera = Camera::new(cgmath::Vector3::new(2.0, 2.0, 2.0), -135.0, -35.0);

    let mut input_state = InputState::default();
    let mut last_frame = std::time::Instant::now();
    let mut camera_focused = false;
    let mut wants_to_grab_cursor = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    if new_size.width > 0 && new_size.height > 0 {
                        app.framebuffer_resized = true;
                    }
                }
                WindowEvent::MouseInput { state: ElementState::Pressed, .. } => {
                    if !camera_focused {
                        wants_to_grab_cursor = true;
                        window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                    }
                }
                WindowEvent::Focused(focused) => {
                    if !focused {
                        camera_focused = false;
                        window.set_fullscreen(None);
                        window.set_cursor_visible(true);
                    }
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        let pressed = input.state == ElementState::Pressed;
                        match key {
                            VirtualKeyCode::W => input_state.forward = pressed,
                            VirtualKeyCode::S => input_state.backward = pressed,
                            VirtualKeyCode::A => input_state.left = pressed,
                            VirtualKeyCode::D => input_state.right = pressed,
                            VirtualKeyCode::Space => input_state.up = pressed,
                            VirtualKeyCode::LShift => input_state.down = pressed,
                            VirtualKeyCode::Escape => {
                                if pressed {
                                    camera_focused = false;
                                    window.set_fullscreen(None);
                                    window.set_cursor_visible(true);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            Event::DeviceEvent { event, .. } => {
                if camera_focused {
                    if let DeviceEvent::MouseMotion { delta } = event {
                        camera.process_mouse(-delta.0 as f32, -delta.1 as f32);
                        let window_size = window.inner_size();
                        let center_x = window_size.width / 2;
                        let center_y = window_size.height / 2;
                        let _ = window.set_cursor_position(winit::dpi::PhysicalPosition::new(center_x, center_y));
                    }
                }
            }
            Event::MainEventsCleared => {
                if wants_to_grab_cursor {
                    camera_focused = true;
                    window.set_cursor_visible(false);
                    wants_to_grab_cursor = false;
                }

                let now = std::time::Instant::now();
                let dt = now.duration_since(last_frame).as_secs_f32();
                last_frame = now;

                if input_state.forward { camera.process_keyboard(CameraMovement::Forward, dt); }
                if input_state.backward { camera.process_keyboard(CameraMovement::Backward, dt); }
                if input_state.left { camera.process_keyboard(CameraMovement::Left, dt); }
                if input_state.right { camera.process_keyboard(CameraMovement::Right, dt); }
                if input_state.up { camera.process_keyboard(CameraMovement::Up, dt); }
                if input_state.down { camera.process_keyboard(CameraMovement::Down, dt); }

                app.draw_frame(&window, &camera);
            }
            _ => {}
        }
    });
}

#[derive(Default)]
struct InputState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}
