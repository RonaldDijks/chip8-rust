use crate::display::Display;

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

pub struct Cpu {
    memory: [u8; 4096],
    display: Display,
    pub pc: u16,
    pub index: u16,
    pub registers: [u8; 16],
    pub stack: [u16; 16],
    pub stack_pointer: usize,
    pub delay_timer: u8,
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
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
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
        if self.delay_timer > 0 {
            self.delay_timer -= 1
        }

        let opcode = self.fetch_opcode();

        self.execute_opcode(opcode);
    }

    fn fetch_opcode(&self) -> u16 {
        let hi = self.memory[self.pc as usize] as u16;
        let lo = self.memory[self.pc as usize + 1] as u16;
        (hi << 8) | lo
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );
        let nnn = (opcode & 0x0FFF) as u16;
        let nn = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as u8;
        let y = nibbles.2 as u8;
        let n = nibbles.3 as u8;

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(),
            (0x1, _, _, _) => self.op_1nnn(nnn),
            (0x2, _, _, _) => self.op_2nnn(nnn),
            (0x3, _, _, _) => self.op_3xnn(x, nn),
            (0x4, _, _, _) => self.op_4xnn(x, nn),
            (0x5, _, _, _) => self.op_5xy0(x, y),
            (0x6, _, _, _) => self.op_6xnn(x, nn),
            (0x7, _, _, _) => self.op_7xnn(x, nn),
            (0x8, _, _, 0x0) => self.op_8xy0(x, y),
            (0x8, _, _, 0x1) => self.op_8xy1(x, y),
            (0x8, _, _, 0x2) => self.op_8xy2(x, y),
            (0x8, _, _, 0x3) => self.op_8xy3(x, y),
            (0x8, _, _, 0x4) => self.op_8xy4(x, y),
            (0x8, _, _, 0x5) => self.op_8xy5(x, y),
            (0x8, _, _, 0x6) => self.op_8xy6(x, y),
            (0x8, _, _, 0x7) => self.op_8xy7(x, y),
            (0x8, _, _, 0xE) => self.op_8xye(x, y),
            (0x9, _, _, 0x0) => self.op_9xy0(x, y),
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xB, _, _, _) => self.op_bnnn(nnn),
            (0xD, _, _, _) => self.op_dxyn(x, y, n),
            (0xF, _, 0x1, 0x5) => self.op_fx15(x),
            (0xF, _, 0x3, 0x3) => self.op_fx33(x),
            (0xF, _, 0x5, 0x5) => self.op_fx55(x),
            (0xF, _, 0x6, 0x5) => self.op_fx65(x),
            _ => {
                panic!("unexpected opcode: {:#06x}", opcode);
            }
        }
    }

    fn op_00e0(&mut self) {
        self.display.clear();
        self.pc += 2;
    }

    fn op_00ee(&mut self) {
        self.stack_pointer -= 1;
        self.pc = self.stack[self.stack_pointer];
    }

    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn op_2nnn(&mut self, nnn: u16) {
        self.stack[self.stack_pointer] = self.pc + 2;
        self.stack_pointer += 1;
        self.pc = nnn;
    }

    fn op_3xnn(&mut self, x: u8, nn: u8) {
        if self.registers[x as usize] == nn {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn op_4xnn(&mut self, x: u8, nn: u8) {
        if self.registers[x as usize] != nn {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn op_5xy0(&mut self, x: u8, y: u8) {
        let x = self.registers[x as usize];
        let y = self.registers[y as usize];
        if x == y {
            self.pc += 2;
        }
        self.pc += 2;
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

    fn op_8xy0(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[y as usize];
        self.pc += 2;
    }

    fn op_8xy1(&mut self, x: u8, y: u8) {
        self.registers[x as usize] |= self.registers[y as usize];
        self.pc += 2;
    }

    fn op_8xy2(&mut self, x: u8, y: u8) {
        self.registers[x as usize] &= self.registers[y as usize];
        self.pc += 2;
    }

    fn op_8xy3(&mut self, x: u8, y: u8) {
        self.registers[x as usize] ^= self.registers[y as usize];
        self.pc += 2;
    }

    fn op_8xy4(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        let (value, overflowed) = vx.overflowing_add(vy);
        self.registers[x as usize] = value;
        self.registers[0xF] = overflowed as u8;
        self.pc += 2;
    }

    fn op_8xy5(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        let (value, overflowed) = vx.overflowing_add(vy);
        self.registers[0xF] = overflowed as u8;
        self.registers[x as usize] = value;
        self.pc += 2;
    }

    fn op_8xy6(&mut self, x: u8, _y: u8) {
        let mut value = self.registers[x as usize];
        let shifted_bit = value & 0xF;
        value >>= 1;
        self.registers[x as usize] = value;
        self.registers[0xF] = shifted_bit;
        self.pc += 2;
    }

    fn op_8xy7(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        let (value, overflowed) = vy.overflowing_add(vx);
        self.registers[0xF] = overflowed as u8;
        self.registers[x as usize] = value;
        self.pc += 2;
    }

    fn op_8xye(&mut self, x: u8, _y: u8) {
        let mut value = self.registers[x as usize];
        let shifted_bit = value >> 7;
        value <<= 1;
        self.registers[x as usize] = value;
        self.registers[0xF] = shifted_bit;
        self.pc += 2;
    }

    fn op_9xy0(&mut self, x: u8, y: u8) {
        let x = self.registers[x as usize];
        let y = self.registers[y as usize];
        if x != y {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn op_annn(&mut self, nnn: u16) {
        self.index = nnn;
        self.pc += 2;
    }

    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = self.registers[0] as u16 + nnn;
    }

    fn op_dxyn(&mut self, x: u8, y: u8, n: u8) {
        self.registers[0x0f] = 0;
        for byte in 0..n {
            let y = (self.registers[y as usize] as usize + byte as usize) % Display::HEIGHT;
            for bit in 0..8 {
                let x = (self.registers[x as usize] as usize + bit) % Display::WIDTH;
                let color = (self.memory[self.index as usize + byte as usize] >> (7 - bit)) & 1;
                let turned_off = color & self.display.pixels[y][x] as u8;
                self.registers[0x0f] |= turned_off;
                self.display.pixels[y][x] ^= color != 0;
            }
        }
        self.pc += 2;
    }

    fn op_fx15(&mut self, x: u8) {
        self.delay_timer = self.registers[x as usize];
        self.pc += 2;
    }

    fn op_fx33(&mut self, x: u8) {
        let idx = self.index as usize;
        let addr = x as usize;
        self.memory[idx] = self.registers[addr] / 100;
        self.memory[idx + 1] = (self.registers[addr] % 100) / 10;
        self.memory[idx + 1] = self.registers[addr] % 10;
        self.pc += 2;
    }

    fn op_fx55(&mut self, x: u8) {
        for offset in 0..=x {
            let addr = self.index + offset as u16;
            self.memory[addr as usize] = self.registers[offset as usize];
        }
        self.pc += 2;
    }

    fn op_fx65(&mut self, x: u8) {
        for offset in 0..=x {
            let addr = self.index + offset as u16;
            self.registers[offset as usize] = self.memory[addr as usize];
        }
        self.pc += 2;
    }
}
