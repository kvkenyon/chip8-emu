extern crate sdl2;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::{Window, WindowContext};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::process;
use std::time::{Duration, Instant};
use std::{env, usize};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
const PITCH: u32 = DISPLAY_WIDTH as u32 * 4;

const START_ADDRESS: usize = 0x200;

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

#[derive(Debug)]
pub struct Config {
    pub file_path: String,
    pub video_scale_factor: u32,
    pub cycle_delay: u32,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didnt get a file path"),
        };

        let video_scale_factor: u32 = match args.next() {
            Some(scale_str) => scale_str.parse().unwrap_or(2),
            None => 2,
        };

        let cycle_delay: u32 = match args.next() {
            Some(delay) => delay.parse().unwrap_or(3),
            None => 3,
        };

        Ok(Config {
            file_path,
            video_scale_factor,
            cycle_delay,
        })
    }
}

#[derive(Debug)]
struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    pc: usize,
    stack: [u16; 16],
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    video: [u32; DISPLAY_SIZE],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut chip8 = Chip8 {
            registers: [0u8; 16],
            memory: [0u8; 4096],
            index: 0,
            pc: START_ADDRESS,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [0; 64 * 32],
        };
        chip8.load_font();
        chip8
    }

    fn fetch(&mut self) -> u16 {
        let msb = self.memory[self.pc as usize];
        let lsb = self.memory[(self.pc + 1) as usize];
        self.pc += 2;
        // or the two bytes to make the instr
        ((msb as u16) << 8) | (lsb as u16)
    }

    fn decode(&mut self, instr: u16) {
        let opcode = instr & 0xF000;
        let x = ((instr & 0x0F00) >> 8) as usize;
        let y = ((instr & 0x00F0) >> 4) as usize;
        let n = instr & 0x000F;
        let nn = instr & 0x00FF;
        let nnn = instr & 0x0FFF;
        match opcode {
            0x0000 => match nn {
                0x00 => {
                    println!("???? opcode={instr:04X}");
                }
                0xE0 => {
                    println!("CLS");
                    self.video = [0; 64 * 32]
                }
                0xEE => {
                    println!("RET");
                    println!(
                        "RET: Restoring PC from stack[{}] = 0x{:03X}",
                        self.sp - 1,
                        self.stack[self.sp - 1]
                    );
                    self.sp -= 1;
                    self.pc = self.stack[self.sp] as usize;
                }
                _ => panic!("Illegal instruction: {instr:04X}"),
            },

            0x1000 => {
                println!("JMP $0x{nnn:03X}");
                self.pc = nnn as usize;
            }
            0x2000 => {
                println!("CALL $0x{nnn:03X}");
                println!(
                    "CALL 0x{:03X}: Saving PC=0x{:03X} to stack[{}]",
                    nnn, self.pc, self.sp
                );
                self.stack[self.sp] = self.pc as u16;
                self.sp += 1;
                self.pc = nnn as usize;
            }
            0x3000 => {
                println!("SE V{x}, $0x{nn:03X}");
                if self.registers[x] as u16 == nn {
                    self.pc += 2;
                }
            }
            0x4000 => {
                println!("SNE V{x}, $0x{nn:03X}");
                if self.registers[x] as u16 != nn {
                    self.pc += 2;
                }
            }
            0x5000 => {
                println!("SE V{x} V{y}");
                if self.registers[x] == self.registers[y] {
                    self.pc += 2;
                }
            }
            0x6000 => {
                println!("LD V{x}, 0x{nn:03X}");
                self.registers[x] = nn as u8;
            }
            0x7000 => {
                // VX := VX + NN,
                println!("ADD V{x}, $0x{nn:03X}");
                let vx = self.registers[x];
                let (result, _) = (vx as u16).overflowing_add(nn);
                self.registers[x] = result as u8;
            }
            0x8000 => match n {
                0x0 => {
                    println!("LD V{x}, V{y}");
                    self.registers[x] = self.registers[y];
                }
                0x1 => {
                    // VX := VX | VY
                    println!("OR V{x}, V{y}");
                    self.registers[x] |= self.registers[y];
                }
                0x2 => {
                    println!("AND V{x}, V{y}");
                    self.registers[x] &= self.registers[y];
                }
                0x3 => {
                    println!("XOR V{x}, V{y}");
                    self.registers[x] ^= self.registers[y];
                }
                0x4 => {
                    println!("ADD V{x}, V{y}");
                    let vx = self.registers[x];
                    let vy = self.registers[y];
                    let (result, overflow) = vx.overflowing_add(vy);
                    self.registers[0xF] = if overflow { 1 } else { 0 };
                    self.registers[x] = result;
                }
                0x5 => {
                    println!("SUB V{x}, V{y}");
                    let vx = self.registers[x];
                    let vy = self.registers[y];
                    self.registers[0xF] = if vx > vy { 1 } else { 0 };
                    let (result, _) = vx.overflowing_sub(vy);
                    self.registers[x] = result;
                }
                0x6 => {
                    println!("SHR V{x}, V{y}");
                    self.registers[0xF] = self.registers[x] & 0x01;
                    self.registers[x] /= 2;
                }
                0x7 => {
                    println!("SUBN V{x}, V{y}");
                    let vx = self.registers[x];
                    let vy = self.registers[y];
                    self.registers[0xF] = if vy > vx { 1 } else { 0 };
                    let (result, _) = vy.overflowing_sub(vx);
                    self.registers[x] = result;
                }
                0xE => {
                    println!("SHL V{x}, V{y}");
                    let vx = self.registers[x];
                    self.registers[0xF] = vx & 0x80;
                    let (r, _) = vx.overflowing_mul(2);
                    self.registers[x] = r;
                }
                _ => panic!("Illegal instruction: {instr}"),
            },
            0x9000 => match n {
                0x0 => {
                    println!("SNE V{x}, V{y}");
                    if self.registers[x] != self.registers[y] {
                        self.pc += 2;
                    }
                }
                _ => panic!("Illegal instruction {instr}"),
            },
            0xA000 => {
                println!("LD I, $0x{nnn:03X}");
                self.index = nnn;
            }
            0xB000 => {
                println!("JMP V0, $0x{nnn:03X}");
                self.pc = (self.registers[0x0] + (nnn as u8)) as usize;
            }
            0xC000 => {
                println!("RND V{x}, $0x{nn:03X}");
                let entropy = rand::rng().random_range(0..255) as u8;
                self.registers[x] = entropy & nn as u8;
            }
            0xD000 => {
                println!("DRW V{x}, V{y}, ${n:02X}");
                let x_coord = self.registers[x] % (DISPLAY_WIDTH as u8);
                let y_coord = self.registers[y] % (DISPLAY_HEIGHT as u8);

                for row in 0..n {
                    let addr = row + self.index;
                    let bits = self.memory[addr as usize];
                    let cy = (y_coord + row as u8) % (DISPLAY_HEIGHT as u8);
                    for col in 0..8 {
                        let cx = (x_coord + col) % (DISPLAY_WIDTH as u8);
                        let sprite_pixel = bits & (0x80 >> col);

                        let screen_pixel_loc = (cy as usize * DISPLAY_WIDTH) + cx as usize;
                        let screen_pixel = self.video[screen_pixel_loc];
                        if sprite_pixel > 0 {
                            if screen_pixel > 0 {
                                self.registers[0xF] = 1;
                            }

                            self.video[screen_pixel_loc] ^= 0xFFFFFFFF;
                        }
                    }
                }
            }
            0xE000 => match nn {
                0x9E => {
                    println!("SKP V{x}");
                    if self.keypad[self.registers[x] as usize] == 1 {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    println!("SKNP V{x}");
                    if self.keypad[self.registers[x] as usize] == 0 {
                        self.pc += 2;
                    }
                }
                _ => panic!("Illegal instruction: {instr}"),
            },
            0xF000 => match nn {
                0x07 => {
                    println!("LD V{x}, DT");
                    self.registers[x] = self.delay_timer;
                }
                0x0A => {
                    println!("LD V{x}, K");
                    if self.keypad[0] == 1 {
                        self.registers[x] = 0;
                    } else if self.keypad[1] == 1 {
                        self.registers[x] = 1;
                    } else if self.keypad[2] == 1 {
                        self.registers[x] = 2;
                    } else if self.keypad[3] == 1 {
                        self.registers[x] = 3;
                    } else if self.keypad[4] == 1 {
                        self.registers[x] = 4;
                    } else if self.keypad[5] == 1 {
                        self.registers[x] = 5;
                    } else if self.keypad[6] == 1 {
                        self.registers[x] = 6;
                    } else if self.keypad[7] == 1 {
                        self.registers[x] = 7;
                    } else if self.keypad[8] == 1 {
                        self.registers[x] = 8;
                    } else if self.keypad[9] == 1 {
                        self.registers[x] = 9;
                    } else if self.keypad[10] == 1 {
                        self.registers[x] = 10;
                    } else if self.keypad[11] == 1 {
                        self.registers[x] = 11;
                    } else if self.keypad[12] == 1 {
                        self.registers[x] = 12;
                    } else if self.keypad[13] == 1 {
                        self.registers[x] = 13;
                    } else if self.keypad[14] == 1 {
                        self.registers[x] = 14;
                    } else if self.keypad[15] == 1 {
                        self.registers[x] = 15;
                    } else {
                        println!("Waiting for keypress...");
                        self.pc -= 2;
                    }
                }
                0x15 => {
                    println!("LD DT, V{x}");
                    self.delay_timer = self.registers[x];
                }
                0x18 => {
                    println!("LD ST, V{x}");
                    self.sound_timer = self.registers[x];
                }
                0x1E => {
                    println!("ADD I, V{x}");
                    self.index += self.registers[x] as u16;
                }
                0x29 => {
                    println!("LD F, V{x}");
                    self.index = (self.registers[x] * 0x05 + 0x050) as u16;
                }
                0x33 => {
                    println!("LD B, V{x}");
                    let vx = self.registers[x];
                    let h = vx / 100;
                    let t = (vx - h * 100) / 10;
                    let o = vx - h * 100 - t * 10;
                    self.memory[self.index as usize] = h;
                    self.memory[(self.index + 1) as usize] = t;
                    self.memory[(self.index + 2) as usize] = o;
                }
                0x55 => {
                    println!("LD [I], V{x}");
                    for reg in 0..=x {
                        self.memory[self.index as usize + reg] = self.registers[reg];
                    }
                }
                0x65 => {
                    println!("LD V{x}, [I]");
                    for reg in 0..=x {
                        self.registers[reg] = self.memory[(self.index as usize) + reg];
                    }
                }
                _ => println!("Illegal instruction: {instr:03X}"),
            },

            _ => panic!("Illegal instruction: {instr:03X}"),
        };
    }

    fn map_scancode_to_chip8_key(scancode: Scancode) -> Option<u8> {
        match scancode {
            Scancode::Num1 => Some(0x1),
            Scancode::Num2 => Some(0x2),
            Scancode::Num3 => Some(0x3),
            Scancode::Num4 => Some(0xC),
            Scancode::Q => Some(0x4),
            Scancode::W => Some(0x5),
            Scancode::E => Some(0x6),
            Scancode::R => Some(0xD),
            Scancode::A => Some(0x7),
            Scancode::S => Some(0x8),
            Scancode::D => Some(0x9),
            Scancode::F => Some(0xE),
            Scancode::Z => Some(0xA),
            Scancode::X => Some(0x0),
            Scancode::C => Some(0xB),
            Scancode::V => Some(0xF),
            _ => None,
        }
    }

    fn load_font(&mut self) {
        println!("[CHIP8] Loading font...");
        // 050–09F
        for (i, addr) in (0x050..0x09f + 1).enumerate() {
            self.memory[addr] = FONT[i];
        }
    }

    pub fn cycle(&mut self) {
        let instr = self.fetch();
        self.decode(instr);
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn load_rom(&mut self, file_path: &String) -> Result<(), Box<dyn Error>> {
        println!("[CHIP8] Loading ROM...");
        let mut file = File::open(file_path)?;
        let bytes_read = file.read(&mut self.memory[0x200..])?;
        println!("[CHIP8] Loaded {bytes_read} bytes from {file_path}.");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn memory_hexdump(&self, start: u16, end: u16) {
        for (i, chunk) in self.memory[start as usize..end as usize]
            .chunks(16)
            .enumerate()
        {
            print!("0x{:03X}: ", (start as usize) + (i * 16));
            for byte in chunk {
                print!("{:02X} ", byte);
            }
            println!();
        }
    }
}

pub struct Renderer {
    canvas: WindowCanvas,
    texture_creator: TextureCreator<WindowContext>, // Store this!
}

impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();

        Ok(Renderer {
            canvas,
            texture_creator,
        })
    }

    pub fn draw(
        &mut self,
        framebuffer: &[u32; DISPLAY_WIDTH * DISPLAY_HEIGHT],
        scale: u32,
    ) -> Result<(), String> {
        // RGBA
        let mut pixels = [0u8; DISPLAY_HEIGHT * PITCH as usize];
        let mut texture = self
            .texture_creator
            .create_texture_streaming(
                PixelFormatEnum::RGBA8888,
                DISPLAY_WIDTH as u32,
                DISPLAY_HEIGHT as u32,
            )
            .map_err(|e| e.to_string())?;

        for (i, &pixel) in framebuffer.iter().enumerate() {
            let color = if pixel != 0 { 255 } else { 0 }; // White or black
            let pixel_start = i * 4;
            pixels[pixel_start] = color; // R
            pixels[pixel_start + 1] = color; // G
            pixels[pixel_start + 2] = color; // B
            pixels[pixel_start + 3] = color; // A
        }

        // Drawing logic here...
        // Update texture with pixel data
        let _ = texture.update(None, &pixels, PITCH as usize);

        // Clear canvas and draw texture scaled up
        self.canvas.set_draw_color(sdl2::pixels::Color::BLACK);
        self.canvas.clear();

        let dst_rect = Rect::new(
            0,
            0,
            DISPLAY_WIDTH as u32 * scale,
            DISPLAY_HEIGHT as u32 * scale,
        );
        self.canvas.copy(&texture, None, Some(dst_rect))?;

        self.canvas.present();
        Ok(())
    }
}

fn main() -> Result<(), String> {
    println!("[CHIP8] Start emulator");

    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    let mut chip8 = Chip8::new();

    chip8.load_rom(&config.file_path).unwrap_or_else(|err| {
        eprintln!("Problem loading ROM @ {}: {err}", &config.file_path);
        process::exit(1);
    });

    println!("[CHIP8] Init window");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "Chip8 Emulator",
            DISPLAY_WIDTH as u32 * config.video_scale_factor,
            DISPLAY_HEIGHT as u32 * config.video_scale_factor,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut renderer = Renderer::new(window)?;

    let mut event_pump = sdl_context.event_pump()?;

    println!("[CHIP8] Start fetch-decode-execute loop");

    let mut last_cycle_time = Instant::now();
    let cycle_delay = Duration::from_millis(config.cycle_delay as u64); // 2ms = 500Hz

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => {
                    // Map the scancode to a CHIP-8 key (0-F)
                    if let Some(chip8_key_index) = Chip8::map_scancode_to_chip8_key(scancode) {
                        // Update your CHIP-8's keypad state
                        chip8.keypad[chip8_key_index as usize] = 1;
                    }
                }
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    // Map the scancode to a CHIP-8 key (0-F)
                    if let Some(chip8_key_index) = Chip8::map_scancode_to_chip8_key(scancode) {
                        // Update your CHIP-8's keypad state
                        chip8.keypad[chip8_key_index as usize] = 0;
                    }
                }
                _ => {}
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

        let current_time = Instant::now();
        let dt = current_time.duration_since(last_cycle_time);

        if dt >= cycle_delay {
            last_cycle_time = current_time;
            chip8.cycle();
            let _ = renderer.draw(&chip8.video, config.video_scale_factor);
        }
        // The rest of the game loop goes here...
    }
    println!("[CHIP8] Exiting...");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]

    fn load_maze_rom_test() {
        let fp = "maze.ch8".to_string();
        let mut chip8 = super::Chip8::new();
        chip8.load_rom(&fp).expect("should load the rom");
        chip8.memory_hexdump(0x200, 0x238);
        assert_eq!(chip8.memory[0x200], 0x60);
        assert_eq!(chip8.memory[0x200], 0x60);
        assert_eq!(chip8.memory[0x201], 0x00);
        assert_eq!(chip8.memory[0x202], 0x61);
        assert_eq!(chip8.memory[0x203], 0x00);
        assert_eq!(chip8.memory[0x204], 0xA2);
        assert_eq!(chip8.memory[0x205], 0x22);
        assert_eq!(chip8.memory[0x210], 0x30);
        assert_eq!(chip8.memory[0x220], 0x20);
        assert_eq!(chip8.memory[0x221], 0x10);
        assert_eq!(chip8.memory[0x230], 0x00);
    }

    #[test]
    fn load_font_test() {
        let mut chip8 = super::Chip8::new();
        chip8.load_font();
        assert_eq!(super::FONT, chip8.memory[0x050..0x09f + 1]);
    }
}
