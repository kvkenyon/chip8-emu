use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::process;

const START_ADDRESS: u16 = 0x200;

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

    pub fn load_rom(&mut self, file_path: &String) -> Result<(), Box<dyn Error>> {
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
}
