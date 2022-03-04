#![feature(array_chunks)]
#![feature(portable_simd)]

use std::time::{Duration, Instant};

use camera::Camera;
use cgmath::Point3;
use eyre::Result;
use raster::Raster;
use renderer::Renderer;
use softbuffer::GraphicsContext;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod camera;
mod obj;
mod raster;
mod renderer;
mod solid;

const WIDTH: usize = 3840;
const HEIGHT: usize = 2160;

fn main() -> Result<()> {
    let portal = obj::load_solid(
        "resources/portal/Portal_C/Portal_C.obj",
        "resources/portal/textures/",
    )?;

    let scene = portal;

    let raster = Raster::new(WIDTH, HEIGHT);
    let camera = Camera::new(Point3::new(0., 20., 4.), 0.5, 0.002);
    let mut renderer = Renderer::new(raster, camera);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;
    window.set_inner_size(PhysicalSize::new(WIDTH as f32, HEIGHT as f32));
    window.set_cursor_visible(false);

    let mut cursor_established = false;
    let mut just_moved = false;
    let mut last_mouse_sample = Instant::now();

    let mut graphics_context =
        unsafe { GraphicsContext::new(window) }.expect("Couldn't initialize graphics context");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(window_id) if window_id == graphics_context.window().id() => {
                renderer.render_solid(&scene);
                let buffer = renderer.img_buf();
                graphics_context.set_buffer(&buffer, WIDTH as u16, HEIGHT as u16);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == graphics_context.window().id() => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        input,
                        is_synthetic: _,
                    },
                window_id,
            } if window_id == graphics_context.window().id() => {
                if let Some(key) = input.virtual_keycode {
                    let camera = renderer.camera();

                    match key {
                        VirtualKeyCode::A => camera.strafe_left(1.),
                        VirtualKeyCode::D => camera.strafe_right(1.),
                        VirtualKeyCode::W => camera.move_forward(1.),
                        VirtualKeyCode::S => camera.move_backward(1.),
                        _ => {}
                    }

                    graphics_context.window().request_redraw();
                }
            }
            #[allow(deprecated)]
            Event::WindowEvent {
                event:
                    WindowEvent::CursorMoved {
                        device_id: _,
                        position,
                        modifiers: _,
                    },
                window_id,
            } if window_id == graphics_context.window().id() && cursor_established => {
                if just_moved {
                    just_moved = false;
                    return;
                }

                let now = Instant::now();
                if now.duration_since(last_mouse_sample) > Duration::from_millis(200) {
                    last_mouse_sample = now;
                } else {
                    return;
                }

                renderer.camera().adjust_look(position.x, position.y);
                graphics_context.window().request_redraw();

                graphics_context
                    .window()
                    .set_cursor_position(PhysicalPosition::new(
                        WIDTH as f32 / 2.,
                        HEIGHT as f32 / 2.,
                    ))
                    .unwrap();
                just_moved = true;
            }
            Event::WindowEvent {
                event: WindowEvent::CursorEntered { device_id: _ },
                window_id,
            } if window_id == graphics_context.window().id() => {
                if !cursor_established {
                    cursor_established = true;
                    last_mouse_sample = Instant::now();
                }

                graphics_context
                    .window()
                    .set_cursor_position(PhysicalPosition::new(
                        WIDTH as f32 / 2.,
                        HEIGHT as f32 / 2.,
                    ))
                    .unwrap();
            }
            _ => {}
        }
    });
}
