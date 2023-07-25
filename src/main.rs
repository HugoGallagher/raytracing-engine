use std::{sync::{mpsc, Mutex, Arc}, ffi::c_void};
use std::thread;

use engine::{math::vec::Vec2, graph::Graph};
use engine::{window, game};
use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle, RawWindowHandle, RawDisplayHandle};
use winit::event::{Event, VirtualKeyCode, WindowEvent, DeviceEvent, ElementState};
use winit::event_loop::{ControlFlow, EventLoop};

pub struct KeyboardMessage {
    key: VirtualKeyCode,
    state: ElementState,
}

pub struct MouseMessage {
    delta: Vec2,
}

pub struct RawWindowDataWrapper {
    window_handle: RawWindowHandle,
    display_handle: RawDisplayHandle,
}

unsafe impl Send for RawWindowDataWrapper {}
unsafe impl Sync for RawWindowDataWrapper {}

#[derive(Copy, Clone)]
struct Data {
    id: u32,
}

#[derive(Copy, Clone)]
struct Info {

}

fn main() {
    // let mut graph = Graph::<Data, Info>::new();

    // graph.add_node("0", Data { id: 0 });
    // graph.add_node("1", Data { id: 1 });
    // graph.add_node("2", Data { id: 2 });
    // graph.add_node("3", Data { id: 3 });
    // graph.add_node("4", Data { id: 4 });
    // graph.add_node("5", Data { id: 5 });
    // graph.add_node("6", Data { id: 6 });
    // graph.add_node("7", Data { id: 7 });
    // graph.add_node("8", Data { id: 8 });

    // graph.add_edge("1", "8", Info {});
    // graph.add_edge("1", "4", Info {});
    // graph.add_edge("4", "6", Info {});
    // graph.add_edge("8", "5", Info {});
    // graph.add_edge("5", "4", Info {});
    // graph.add_edge("4", "7", Info {});
    // graph.add_edge("7", "2", Info {});
    // graph.add_edge("6", "3", Info {});

    // let nodes = graph.breadth_first_forwards("1");
    // for node in nodes {
    //     println!("{}", node.name);
    // }

    // return;

    unsafe {
        let event_loop = EventLoop::new();
        let mut window = window::Window::new(&event_loop);

        let (key_t, key_r) = mpsc::channel::<KeyboardMessage>();
        let (mouse_t, mouse_r) = mpsc::channel::<MouseMessage>();

        let raw_window_data = RawWindowDataWrapper {
            window_handle: window.window.raw_window_handle(),
            display_handle: window.window.raw_display_handle(),
        };

        let game_handle = thread::spawn(move || {
            let raw_window_data_copy = raw_window_data;
            let mut game = game::Game::new(raw_window_data_copy.window_handle, raw_window_data_copy.display_handle, Vec2::new(window.res.0 as f32, window.res.1 as f32));

            let game_should_close = false;

            while !game_should_close {
                match key_r.try_recv() {
                    Ok(msg) => {
                        game.update_key(msg.key, msg.state);
                    },
                    _ => {},
                }

                let mut mouse_msg = mouse_r.try_recv();
                while mouse_msg.is_ok() {
                    game.mouse_delta = game.mouse_delta + mouse_msg.unwrap().delta;
                    mouse_msg = mouse_r.try_recv();
                }

                game.main_loop();
            }
        });

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit;
                },
                Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
                    if input.virtual_keycode.is_some() {
                        if input.virtual_keycode.unwrap() == VirtualKeyCode::Escape {
                            window.window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
                            window.window.set_cursor_visible(true);
                        }

                        key_t.send(KeyboardMessage {
                            key: input.virtual_keycode.unwrap(),
                            state: input.state,
                        }).unwrap();
                    }
                },
                Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                    if window.focused {
                        mouse_t.send(MouseMessage { delta: Vec2::new(delta.0 as f32, delta.1 as f32) }).unwrap();
                    }
                }
                Event::WindowEvent { event: WindowEvent::Focused(f), .. } => {
                    window.focused = f;
                },
                _ => ()
            };
        });
    }
}