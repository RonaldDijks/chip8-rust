use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const PC_START: usize = 0x200;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

struct Display {
    pixels: [[bool; Self::WIDTH]; Self::HEIGHT],
}

impl Display {
    pub const WIDTH: usize = 64;
    pub const HEIGHT: usize = 32;

    pub fn new() -> Self {
        Self {
            pixels: [[false; Self::WIDTH]; Self::HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        for row in self.pixels.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = false;
            }
        }
    }
}

struct DisplayRenderer;

struct Cpu {
    memory: [u8; 4096],
    display: Display,
    pc: u16,
    index: u16,
    registers: [u8; 16],
}

impl Cpu {
    pub fn new() -> Self {
        let mut memory = [0; 4096];
        for (index, pixel) in FONT.iter().enumerate() {
            memory[index] = *pixel;
        }

        Self {
            memory,
            display: Display::new(),
            pc: PC_START as u16,
            index: 0,
            registers: [0; 16],
        }
    }

    pub fn load(&mut self, rom: &[u8]) {
        for (i, &byte) in rom.iter().enumerate() {
            let addr = 0x200 + i;
            if addr < 4096 {
                self.memory[addr] = byte;
            } else {
                break;
            }
        }
    }

    pub fn get_display(&self) -> &Display {
        &self.display
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch_opcode();

        self.execute_opcode(opcode);
    }

    fn fetch_opcode(&self) -> u16 {
        let hi = self.memory[self.pc as usize] as u16;
        let lo = self.memory[self.pc as usize + 1] as u16;
        (hi << 8) | lo
    }

    fn execute_opcode(&mut self, opcode: u16) {
        println!("{:#06x}", opcode);

        let nibbles = (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            (opcode & 0x000F),
        );
        let nnn = (opcode & 0x0FFF) as u16;
        let nn = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as u8;
        let y = nibbles.2 as u8;
        let n = nibbles.3 as u8;

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),
            (0x1, _, _, _) => self.op_1nnn(nnn),
            (0x6, _, _, _) => self.op_6xnn(x, nn),
            (0x7, _, _, _) => self.op_7xnn(x, nn),
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xD, _, _, _) => self.op_dxyn(x as usize, y as usize, n as usize),
            _ => {
                panic!("unexpected opcode")
            }
        }
    }

    fn op_00e0(&mut self) {
        self.display.clear();
        self.pc += 2;
    }

    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn op_6xnn(&mut self, x: u8, nn: u8) {
        self.registers[x as usize] = nn;
        self.pc += 2;
    }

    fn op_7xnn(&mut self, x: u8, nn: u8) {
        let vx = self.registers[x as usize] as u16;
        let value = nn as u16;
        let result = vx + value;
        self.registers[x as usize] = result as u8;
        self.pc += 2;
    }

    fn op_annn(&mut self, nnn: u16) {
        self.index = nnn;
        self.pc += 2;
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) {
        println!("x: {}, y: {}, n: {}", x, y, n);
        self.registers[0x0f] = 0;
        for byte in 0..n {
            let y = (self.registers[y] as usize + byte) % Display::HEIGHT;
            for bit in 0..8 {
                let x = (self.registers[x] as usize + bit) % Display::WIDTH;
                let color = (self.memory[self.index as usize + byte] >> (7 - bit)) & 1;
                let turned_off = color & self.display.pixels[y][x] as u8;
                self.registers[0x0f] |= turned_off;
                self.display.pixels[y][x] ^= color != 0;
            }
        }
        self.pc += 2;
    }
}

impl DisplayRenderer {
    pub fn draw(&self, display: &Display, output_frame: &mut [u8]) {
        for (i, pixel) in output_frame.chunks_exact_mut(4).enumerate() {
            let x = (i % Display::WIDTH) as usize;
            let y = (i / Display::WIDTH) as usize;
            let is_on = display.pixels[y][x];
            let color = if is_on {
                [0xFF, 0xFF, 0xFF, 0xFF]
            } else {
                [0x00, 0x00, 0x00, 0x00]
            };
            pixel.copy_from_slice(&color);
        }
    }
}

fn main() {
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

    let rom = include_bytes!("./../roms/ibm_logo.ch8");
    let mut cpu = Cpu::new();
    cpu.load(rom);
    let renderer = DisplayRenderer;

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            renderer.draw(cpu.get_display(), pixels.get_frame());
            cpu.tick();

            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            window.request_redraw();
        }
    })
}
