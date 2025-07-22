const START_ADDRESS: u16 = 0x200;

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
            registers: [0; 16],
            memory: [0; 4096],
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
}

fn main() {
    println!("[CHIP8] Start emulator");

    let chip8 = Chip8::new();
    println!("reg: {:?}", chip8.registers);
    println!("mem: {:?}", chip8.memory);
    println!("index: {:?}", chip8.index);
    println!("pc: {:?}", chip8.pc);
    println!("sp: {:?}", chip8.sp);
    println!("stack: {:?}", chip8.stack);
    println!("delay_timer: {:?}", chip8.delay_timer);
    println!("sound_timer: {:?}", chip8.sound_timer);
    println!("keypad: {:?}", chip8.keypad);
    println!("video: {:?}", chip8.video[0]);
    println!("op_code: {:?}", chip8.op_code);
    println!("[CHIP8] Exiting...");
}
