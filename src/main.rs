use engine::math::vec::Vec2;
use engine::{window, game};
use winit::event::{Event, VirtualKeyCode, WindowEvent, DeviceEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    unsafe {
        let event_loop = EventLoop::new();
        let mut window = window::Window::new(&event_loop);
        let mut game = game::Game::new(&window, Vec2::new(window.res.0 as f32, window.res.1 as f32));

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit;
                },
                Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
                    if input.virtual_keycode.is_some() {
                        if input.virtual_keycode.unwrap() == VirtualKeyCode::Escape {
                            window.window.set_cursor_grab(false).unwrap();
                            window.window.set_cursor_visible(true);
                        }

                        game.update_key(input.virtual_keycode.unwrap(), input.state);
                    }
                },
                Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                    if window.focused {
                        game.mouse_delta = Vec2::new(delta.0 as f32, delta.1 as f32);
                    }
                }
                Event::WindowEvent { event: WindowEvent::Focused(f), .. } => {
                    window.focused = f;
                },
                Event::MainEventsCleared => {
                    game.main_loop();
                },
                _ => ()
            };
        });
    }
}