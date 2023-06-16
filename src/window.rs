use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};

pub struct Window {
    pub window: winit::window::Window,
    pub focused: bool,
    pub res: (u32, u32),
}

impl Window {
    pub fn new(el: &EventLoop<()>) -> Window {
        let window = WindowBuilder::new()
            .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
            .with_title("Raytracer")
            .build(el)
            .unwrap();

        //window.set_cursor_grab(true).unwrap();
        //window.set_cursor_visible(false);

        Window {
            window: window,
            focused: true,
            res: (1280, 720),
        }
    }
}