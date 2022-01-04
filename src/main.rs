use cpu::Cpu;
use display::Display;
use gui::Gui;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use renderer::DisplayRenderer;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use structopt::StructOpt;
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

mod cpu;
mod display;
mod gui;
mod renderer;

#[derive(Debug, StructOpt)]
#[structopt(name = "chip-8", about = "A chip-8 emulator.")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(Display::WIDTH as u32, Display::HEIGHT as u32);
        WindowBuilder::new()
            .with_title("Chip 8")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(
            Display::WIDTH as u32,
            Display::HEIGHT as u32,
            surface_texture,
        )
        .unwrap()
    };

    let rom = std::fs::read(opt.input).unwrap();
    let mut cpu = Cpu::new();
    cpu.load(&rom);
    let renderer = DisplayRenderer;

    let mut gui = Gui::new(&window, &pixels);

    let mut last_render = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            renderer.draw(cpu.get_display(), pixels.get_frame());

            gui.prepare(&window).expect("gui.prepare() failed");

            let render_result = pixels.render_with(|encoder, render_target, context| {
                context.scaling_renderer.render(encoder, render_target);
                gui.render(&window, encoder, render_target, context, &cpu)?;

                Ok(())
            });

            if render_result
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        gui.handle_event(&window, &event);

        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }
        }

        let now = Instant::now();
        if (now - last_render) > Duration::from_secs_f32(1. / 15.) {
            last_render = now;
            cpu.tick();
        }

        window.request_redraw();
    })
}
