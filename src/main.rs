use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::process;

const START_ADDRESS: u16 = 0x200;

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
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didnt get a file path"),
        };

        Ok(Config { file_path })
    }
}

#[derive(Debug)]
struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    pc: u16,
    stack: [u8; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    video: [u32; 64 * 32],
    op_code: u16,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
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
            op_code: 0,
        }
    }

    pub fn fetch(&mut self) -> u16 {
        let msb = self.memory[self.pc as usize];
        let lsb = self.memory[(self.pc + 1) as usize];
        self.pc += 2;
        ((msb as u16) << 8) | (lsb as u16)
    }

    pub fn decode(&self, instr: u16) {
        let opcode = instr & 0xF000;
        let x = ((instr & 0x0F00) >> 8) as usize;
        let y = ((instr & 0x00F0) >> 4) as usize;
        let n = instr & 0x000F;
        let nn = instr & 0x00FF;
        let nnn = instr & 0x0FFF;
        match opcode {
            0x00E0 => println!("CLS"),
            0x00EE => println!("RET"),
            0x1000 => println!("JMP $0x{nnn:03X}"),
            0x2000 => println!("CALL $0x{nnn:03X}"),
            0x3000 => println!("SE V{x}, $0x{nn:03X}"),
            0x4000 => println!("SNE V{x}, $0x{nn:03X}"),
            0x5000 => println!("SNE V{x} V{y}"),
            0x6000 => println!("LD V{x}, $0x{nn:03X}"),
            0x7000 => println!("ADD V{x}, $0x{nn:03X}"),
            0x8000 => match n {
                0x0 => println!("LD V{x}, V{y}"),
                0x1 => println!("OR V{x}, V{y}"),
                0x2 => println!("AND V{x}, V{y}"),
                0x3 => println!("XOR V{x}, V{y}"),
                0x4 => println!("ADD V{x}, V{y}"),
                0x5 => println!("SUB V{x}, V{y}"),
                0x6 => println!("SHR V{x}, V{y}"),
                0x7 => println!("SUBN V{x}, V{y}"),
                0xE => println!("SHL V{x}, V{y}"),
                _ => panic!("Illegal instruction: {instr}"),
            },
            0x9000 => match n {
                0x0 => println!("SNE V{x}, V{y}"),
                _ => panic!("Illegal instruction {instr}"),
            },
            0xA000 => println!("LD I, $0x{nnn:03X}"),
            0xB000 => println!("JMP V0, $0x{nnn:03X}"),
            0xC000 => println!("RND V{x}, $0x{nn:03X}"),
            0xD000 => println!("DRW V{x}, V{y}, ${n:02X}"),
            0xE000 => match nn {
                0x9E => println!("SKP V{x}"),
                0xA1 => println!("SKNP V{x}"),
                _ => panic!("Illegal instruction: {instr}"),
            },
            0xF000 => match nn {
                0x07 => println!("LD V{x}, DT"),
                0x0A => println!("LD V{x}, K"),
                0x15 => println!("LD DT, V{x}"),
                0x18 => println!("LD ST, V{x}"),
                0x1E => println!("ADD I, V{x}"),
                0x29 => println!("LD F, V{x}"),
                0x33 => println!("LD B, V{x}"),
                0x55 => println!("LD [I], V{x}"),
                0x65 => println!("LD V{x}, [I]"),
                _ => println!("Illegal instruction: {instr}"),
            },

            _ => panic!("Illegal instruction: {instr}"),
        };
    }

    pub fn execute(&self) {}

    pub fn load_font(&mut self) {
        println!("[CHIP8] Loading font...");
        // 050–09F
        for (i, addr) in (0x050..0x09f + 1).enumerate() {
            self.memory[addr] = FONT[i];
        }
    }

    pub fn load_rom(&mut self, file_path: &String) -> Result<(), Box<dyn Error>> {
        println!("[CHIP8] Loading ROM...");
        let mut file = File::open(file_path)?;
        let bytes_read = file.read(&mut self.memory[0x200..])?;
        println!("[CHIP8] Loaded {bytes_read} bytes from {file_path}.");
        Ok(())
    }

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

fn main() {
    println!("[CHIP8] Start emulator");

    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    let mut chip8 = Chip8::new();
    println!("reg: {:?}", chip8.registers);
    println!("mem: {:?}", chip8.memory[0x200]);
    println!("index: {:?}", chip8.index);
    println!("pc: {:?}", chip8.pc);
    println!("sp: {:?}", chip8.sp);
    println!("stack: {:?}", chip8.stack);
    println!("delay_timer: {:?}", chip8.delay_timer);
    println!("sound_timer: {:?}", chip8.sound_timer);
    println!("keypad: {:?}", chip8.keypad);
    println!("video: {:?}", chip8.video[0]);
    println!("op_code: {:?}", chip8.op_code);

    chip8.load_rom(&config.file_path).unwrap_or_else(|err| {
        eprintln!("Problem loading ROM @ {}: {err}", &config.file_path);
        process::exit(1);
    });

    chip8.memory_hexdump(0x200, 0x238);

    chip8.load_font();

    chip8.memory_hexdump(0x050, 0x09F + 1);

    println!("[CHIP8] Start fetch-decode-execute loop");
    for _ in 0..10 {
        let instr = chip8.fetch();
        //let decoded_inster = chip8.decode(instr);
        //chip8.execute(decoded_inster);
        chip8.decode(instr);
    }

    println!("[CHIP8] Exiting...");
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
