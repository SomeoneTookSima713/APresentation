use winit::dpi::LogicalSize;
use winit::event::{ Event, WindowEvent, KeyEvent, ElementState };
use winit::event_loop::EventLoop;
use winit::keyboard::{ Key, NamedKey };
use winit::window::{ Window, WindowBuilder };

mod util;

pub fn run() -> anyhow::Result<()> {
    let (event_loop, window) = init_window()?;

    event_loop.run(move |event, event_loop| {
        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput { event: KeyEvent { logical_key: Key::Named(NamedKey::Escape), state: ElementState::Pressed, .. }, .. } => {
                        event_loop.exit();
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    })?;

    Ok(())
}

fn init_window() -> anyhow::Result<(EventLoop<()>, Window)> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::<i32>::from(util::consts::WINDOW_SIZE))
        .with_resizable(true)
        .with_title(util::consts::WINDOW_TITLE)
        .with_window_icon(Some(winit::window::Icon::from_rgba(Vec::from(*util::consts::ICON_DATA), util::consts::ICON_WIDTH as u32, util::consts::ICON_HEIGHT as u32)?))
        .build(&event_loop)?;
    Ok((event_loop, window))
}