use core::panic;
use std::sync::{Arc, Mutex};

const MEM_SIZE: usize = 0xFFF + 1; // 4KiB

// it is apparently popular to put the font at 050–09F ... so I will do that as well
const FONT_START: usize = 0x50;
const FONT_CHARACTER_BYTES: usize = 5;

// for compability with older programs
const PROGRAM_START: usize = 0x200;

pub const DEFAULT_FONT: [u8; 80] = [
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

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

///
/// The display is monochrome. It should be 64 pixels wide and 32 pixels tall.
/// For further details on the chip8 display see: <https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#display>
pub trait Display {
    ///
    /// Modifies the screen starting at display position (x,y) with sprite.
    ///
    /// If any pixel is turned off, the function must return true, otherwise false.
    ///
    /// The bits of the sprite are XOR'd with the bits on the screen. For further detail see: <https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#dxyn-display>
    /// # Arguments
    /// * 'sprite' - sprite used to modify the display
    /// * 'n' - height of sprite
    /// * 'x' - x display start position
    /// * 'y' - y display start position
    ///  
    fn modify(&mut self, sprite: &[u8], n: u8, x: u8, y: u8) -> bool;

    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn clear(&mut self);
}

///
/// The chip8 keypad is hexdecimal. It contains buttons for 0-9 and A-F.
///
/// For further details see: <https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#keypad>
pub trait Keypad {
    ///
    /// Returns `Some<_>` if a key is *currently* pressed, `None` otherwise. The Some contains the pressed key as an `u8` (0x0 .. 0xF)
    fn get_pressed_key(&self) -> Option<u8>;
}

pub trait Beeper {
    ///
    /// Starts the Beeper. The Beeper counter initialized by time must be decremented 60 times per second.
    ///
    /// The Beeper stops beeping when the internal counter reaches zero.
    ///
    /// # Arguments
    ///
    /// * 'time' - value that the internal counter is initialized with
    ///
    fn start(&mut self, time: u8);
}

/// The chip8 timer is a 8-Bit timer that decrements its internal value 60 times a second. Chip8 has two timers.
/// The sound timer should be implemented with the [Beeper] trait. This trait is intended for the delay timer.
///
/// For further details see: <https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#timers.>
pub trait Timer {
    fn set(&mut self, val: u8);
    fn get(&self) -> u8;
}

// choosing trait objects to make gui stuff easier
// making everything threadsafe so that IO stuff can run in different threads
pub struct State {
    memory: Vec<u8>,
    // u16 should be enough for the usual 4k, but usize should be better for indexing the memory vector
    pc: usize,
    index_reg: u16,
    stack: Vec<usize>,
    // the 16 general purpose registers
    gp_registers: [u8; 16],

    rng: RngWrapper,

    display: Arc<Mutex<dyn Display>>,
    delay_timer: Arc<Mutex<dyn Timer>>,
    sound_timer: Arc<Mutex<dyn Beeper>>,
    keypad: Arc<Mutex<dyn Keypad>>,
}

// wrapper for rng, rand does not work (easily?) with wasm.
// TODO support different generators depending on platform
struct RngWrapper {
    generator: rand::rngs::ThreadRng,
}

impl RngWrapper{
    fn new() -> Self{
        Self{generator: rand::thread_rng()}
    }

    fn generate_random_byte(&mut self) -> u8{
        rand::Rng::gen(&mut self.generator)
    }
}
// Some mock structs for testing and debugging
// ----------------------------------------------------------------
pub struct DebugDisplay {
    pub ret: bool,
    pub width: usize,
    pub height: usize,
}

impl Display for DebugDisplay {
    #[allow(unused_variables)]
    fn modify(&mut self, sprite: &[u8], n: u8, x: u8, y: u8) -> bool {
        self.ret
    }

    fn height(&self) -> usize {
        self.height
    }

    fn width(&self) -> usize {
        self.width
    }

    fn clear(&mut self) {
        return;
    }
}

pub struct DebugKeypad {
    pub currently_pressed: Option<u8>,
}
impl Keypad for DebugKeypad {
    fn get_pressed_key(&self) -> Option<u8> {
        self.currently_pressed
    }
}

pub struct DebugBeeper {
    pub value: u8,
}
impl Beeper for DebugBeeper {
    fn start(&mut self, time: u8) {
        self.value = time;
    }
}

pub struct DebugTimer {
    pub value: u8,
}
impl Timer for DebugTimer {
    fn get(&self) -> u8 {
        self.value
    }

    fn set(&mut self, val: u8) {
        self.value = val;
    }
}

// ----------------------------------------------------------------

// A proper display implementation
// ----------------------------------------------------------------

/// This struct implements the Display trait. Modify only affects the display vec. The display is 64x32 pixels.
pub struct DisplayBuffer {
    pub display: Vec<bool>,
    display_width: usize,
    display_height: usize,
}

impl DisplayBuffer {
    pub fn new() -> Self {
        let display_width = 64;
        let display_height = 32;

        let display = vec![false; display_width * display_height];

        Self {
            display,
            display_width,
            display_height,
        }
    }

    pub fn get_width(&self) -> usize {
        self.display_width
    }

    pub fn get_height(&self) -> usize {
        self.display_height
    }
}

// TODO: check if the result may be reversed for the display values
fn u8_to_bool_array(byte: u8) -> [bool; 8] {
    let mut bool_array = [false; 8];
    for i in 0..=7 {
        let mask = 0b10000000 >> i;
        bool_array[i] = (byte & mask) != 0;
    }
    // kinda cool that this works in rust (returning array). Probably just copy
    bool_array
}

impl Display for DisplayBuffer {
    fn modify(&mut self, sprite: &[u8], n: u8, x: u8, y: u8) -> bool {
        // must be set to true if a pixel of the display is turned off
        let mut result_flag = false;

        // should wrap, x = 5 should be the same as x = 68
        let actual_x = x % self.display_width as u8;
        let actual_y = y % self.display_height as u8;

        // sprites should be clipped
        // sprites are 8 pixels wide (each u8 of the sprite) and n pixels tall
        // the sprite just XORs each bit with the corresponding display pixel

        for line in 0..n {
            
            let line_bools = u8_to_bool_array(sprite[line as usize]);
            //println!("\t{:?}", line_bools);
            /*
            line_bools.clone().map(|i| {
                if i{
                    print!("█");
                } else {
                    print!(" ");
                }
            });

            println!("");
            */

            if (actual_y + line) as usize >= self.display_height {
                // sprite should clip so we are finished
                return result_flag;
            }

            for (i, b) in line_bools.iter().enumerate() {
                // drawing should clip
                if actual_x as usize + i < self.display_width {
                    let index =
                        actual_x as usize + i + self.display_width * (line + actual_y) as usize;
                    let old = self.display[index];
                    // note that != is the same as a logical XOR
                    self.display[index] = self.display[index] != *b;

                    // if the bit was set a pixel was flipped
                    if *b  && old{
                        result_flag = true;
                    }
                }
            }
        }
        result_flag
    }

    fn height(&self) -> usize {
        self.display_height
    }

    fn width(&self) -> usize {
        self.display_width
    }

    fn clear(&mut self) {
        self.display.fill(false);
    }
}
// ----------------------------------------------------------------

impl State {
    pub fn new(
        display: Arc<Mutex<dyn Display + Send>>,
        delay_timer: Arc<Mutex<dyn Timer + Send>>,
        sound_timer: Arc<Mutex<dyn Beeper + Send>>,
        keypad: Arc<Mutex<dyn Keypad + Send>>,
    ) -> Self {

        State {
            memory: vec![0; MEM_SIZE],
            pc: 0,
            index_reg: 0,
            stack: Vec::new(),
            gp_registers: [0; 16],
            rng: RngWrapper::new(),
            display,
            delay_timer,
            sound_timer,
            keypad,
        }
    }

    pub fn initialize(&mut self, program: &[u8], font: &[u8]) {
        // load program into memory
        for i in 0..program.len() {
            self.memory[PROGRAM_START + i] = program[i];
        }

        self.pc = PROGRAM_START;

        for i in 0..font.len() {
            self.memory[FONT_START + i] = font[i];
        }
    }

    // execute the next instruction located at pc
    pub fn execute(&mut self) {
        // fetch, chip8 uses big endian
        let upper = self.memory[self.pc];
        let lower = self.memory[self.pc+1];

        let instruction = (upper as u16) << 8 | (lower as u16);
        // keep in mind that the pc is incremented here, important for some instructions
        self.pc += 2;

        //println!("{:#06x}", instruction);
        // Decode
        let instruction  = Instruction::decode(instruction);

        //println!("{:?}", instruction);
        

        match instruction {
            Instruction::Cls => self.display.lock().unwrap().clear(),
            Instruction::Rts => self.pc = self.stack.pop().unwrap(),
            Instruction::Jump{nnn} => self.pc = nnn as usize,
            Instruction::Call { nnn } => {
                self.stack.push(self.pc);
                self.pc = nnn as usize;
            },
            Instruction::SkipEqConst { x, nn } => if self.gp_registers[x as usize] == nn {self.pc += 2;},
            Instruction::SkipNeqConst { x, nn } => if self.gp_registers[x as usize] != nn {self.pc += 2;},
            Instruction::SkipEq { x, y } => if self.gp_registers[x as usize] == self.gp_registers[y as usize] {self.pc += 2},
            Instruction::MovConst { x, nn } => self.gp_registers[x as usize] = nn,
            Instruction::AddConst { x, nn } => self.gp_registers[x as usize] = (self.gp_registers[x as usize] as u16 + nn as u16) as u8, // properly handle overflow, as u8 should truncate
            Instruction::Mov { x, y } => self.gp_registers[x as usize] = self.gp_registers[y as usize],
            Instruction::Or { x, y } => self.gp_registers[x as usize] = self.gp_registers[x as usize] | self.gp_registers[y as usize] as u8,
            Instruction::And { x, y } => self.gp_registers[x as usize] &= self.gp_registers[y as usize],
            Instruction::Xor { x, y } => self.gp_registers[x as usize] ^= self.gp_registers[y as usize],
            Instruction::Add { x, y } => {
                let sum = self.gp_registers[x as usize] as u16 + self.gp_registers[y as usize] as u16;
                if sum > 0xFF{
                    self.gp_registers[0xF] = 1;
                } else {
                    self.gp_registers[0xF] = 0;
                }
                self.gp_registers[x as usize] = sum as u8;
            },
            Instruction::SubXY { x, y } => {
                let x_val:u8 = self.gp_registers[x as usize];
                let y_val:u8 = self.gp_registers[y as usize];


                if x_val > y_val{
                    self.gp_registers[0xF] = 1;
                    self.gp_registers[x as usize] = x_val - y_val;
                } else {
                    self.gp_registers[0xF] = 0;
                    // TODO: check if this is the right behavior
                    self.gp_registers[x as usize] = 0xFF - (y_val - x_val);
                }
            },
            Instruction::RightShift { x, y: _ } => {
                self.gp_registers[0xF] = self.gp_registers[x as usize] & 0x01;
                self.gp_registers[x as usize] = self.gp_registers[x as usize] >> 1;
            },
            Instruction::SubYX { x, y } =>{
                let x_val:u8 = self.gp_registers[x as usize];
                let y_val:u8 = self.gp_registers[y as usize];


                if y_val > x_val{
                    self.gp_registers[0xF] = 1;
                    self.gp_registers[x as usize] = y_val - x_val;
                } else {
                    self.gp_registers[0xF] = 0;
                    // TODO: check if this is the right behavior
                    self.gp_registers[x as usize] = 0xFF - (x_val - y_val);
                    
                }
            },
            Instruction::LeftShift { x, y: _ } => {
                self.gp_registers[0xF] = self.gp_registers[x as usize] & 0x80;
                self.gp_registers[x as usize] = self.gp_registers[x as usize] << 1;
            },
            Instruction::SkipNeq { x, y } => {
                if self.gp_registers[x as usize] != self.gp_registers[y as usize] {
                    self.pc += 2;
                }
            },
            Instruction::MovI { nnn } => self.index_reg = nnn,
            Instruction::JumpIndexed { nnn } => self.pc = nnn as usize + self.gp_registers[0] as usize,
            
            // TODO: Rand, implement own rng, so that it is easier to compile to wasm later (rand is for some reason not wasm compatible? Better: just use wbg_rand)
            Instruction::Rand { x, nn } => self.gp_registers[x as usize] = self.rng.generate_random_byte() & nn,

            Instruction::Draw { x, y, n } => {
                let res = self.display.lock().unwrap().modify(&self.memory[(self.index_reg as usize)..((self.index_reg+(n as u16)) as usize)], n, self.gp_registers[x as usize], self.gp_registers[y as usize]);
                if res{
                    self.gp_registers[0xF] = 1;
                } else {
                    self.gp_registers[0xF] = 0;
                }
            },

            Instruction::SkipKeyEq { x } => {
                let key = self.keypad.lock().unwrap().get_pressed_key();
                if let Some(k) = key {
                    if k == self.gp_registers[x as usize]{
                        self.pc += 2;
                    }
                }
            },

            Instruction::SkipKeyNeq { x } => {
                let key = self.keypad.lock().unwrap().get_pressed_key();
                if key.is_none() {
                    self.pc += 2;
                } else if let Some(k) = key {
                    if k != self.gp_registers[x as usize] {
                        self.pc += 2;
                    }
                }
            }
            Instruction::GetDelayTimer { x } => self.gp_registers[x as usize] = self.delay_timer.lock().unwrap().get(),
            // just reexecutes the instruction if no key was pressed
            Instruction::WaitKey { x } => {
                let key = self.keypad.lock().unwrap().get_pressed_key();
                if let Some(k) = key {
                    self.gp_registers[x as usize] = k;
                } else {
                    self.pc -= 2;
                }
            },
            Instruction::SetDelayTimer { x } => self.delay_timer.lock().unwrap().set(self.gp_registers[x as usize]),
            Instruction::SetSoundTimer { x } => self.sound_timer.lock().unwrap().start(self.gp_registers[x as usize]),
            Instruction::AddI { x } => self.index_reg = (self.index_reg + self.gp_registers[x as usize] as u16) & 0x0FFF,
            // just consider the lower nibble of the register
            Instruction::SetFontI { x } => self.index_reg = (FONT_START + FONT_CHARACTER_BYTES * (self.gp_registers[x as usize] & 0x0F) as usize) as u16,
            Instruction::BCD { x } => {
                let mut x_val = self.gp_registers[x as usize];
                self.memory[((self.index_reg + 2) & 0x0FFF) as usize] = x_val % 10;
                x_val /= 10;
                self.memory[(self.index_reg + 1 & 0x0FFF) as usize] = x_val % 10;
                x_val /= 10;
                self.memory[self.index_reg as usize] = x_val;
                
            },
            Instruction::RegDump { x } => {
                for i in 0..=(x as usize){
                    self.memory[(self.index_reg as usize + i ) & 0x0FFF] = self.gp_registers[i];
                }
            },
            Instruction::RegLoad { x } => {
                for i in 0..=(x as usize){
                    self.gp_registers[i] = self.memory[(self.index_reg as usize + i ) & 0x0FFF];
                }
            },

            Instruction::Invalid =>{
                println!("{:#04x} {:#04x}", upper, lower);
                panic!("Not yet implemented");
            } 
        }
    }
}



// Mnemonics are (mostly) taken from: http://www.emulator101.com/chip-8-instruction-set.html
// also https://en.wikipedia.org/wiki/CHIP-8
// X: second nibble of instruction. Used to look up one of the 16 registers
// Y: third nibble of instruction. Used to look up one of the 16 registers
// N: The *fourth* nibble
// NN: second byte, immediate 8-bit number
// NNN: second, third and fourth nibble, immediate 12-bit address
#[derive(Debug)]
pub enum Instruction {
    // 0NNN, Instruction 0NNN calls a machine code routine (RCA 1802 for COSMAC VIP), I won't implement this instruction
    // use Invalid for this Instruction
    Invalid,
    // 00E0, clear screen
    Cls,
    // 00EE, return from subroutine
    Rts,
    // 1NNN, absolute jump to NNN
    Jump { nnn: u16 },
    // 2NNN, jump to subroutine at NNN (push address to stack, change pc)
    Call { nnn: u16 },
    // 3XNN, skip next instruction if Vx equals NN
    SkipEqConst { x: u8, nn: u8 },
    // 4XNN, skip next instruction if Vx does not equal NN
    SkipNeqConst { x: u8, nn: u8 },
    // 5XY0, skips the next instruction if VX equals VY
    SkipEq { x: u8, y: u8 },
    // 6XNN, Sets VX to NN.
    MovConst { x: u8, nn: u8 },
    // 7XNN, Adds NN to VX (carry flag is not changed)
    AddConst { x: u8, nn: u8 },
    // 8XY0, Sets VX to the value of VY.
    Mov { x: u8, y: u8 },
    // 8XY1, Sets VX to VX or VY. (bitwise OR operation)
    Or { x: u8, y: u8 },
    // 8XY2, Sets VX to VX and VY. (bitwise AND operation)
    And { x: u8, y: u8 },
    // 8XY3, Sets VX to VX xor VY
    Xor { x: u8, y: u8 },
    // 8XY4, Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there is not.
    Add { x: u8, y: u8 },
    // 8XY5, VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there is not.
    SubXY { x: u8, y: u8 },
    // 8XY6, Stores the least significant bit of VX in VF and then shifts VX to the right by 1 (ambiguous see chip8 guide)
    RightShift { x: u8, y: u8 },
    // 8XY7, Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not.
    SubYX { x: u8, y: u8 },
    // 8XYE, Stores the most significant bit of VX in VF and then shifts VX to the left by 1
    LeftShift { x: u8, y: u8 },
    // 9XY0, Skips the next instruction if VX does not equal VY
    SkipNeq { x: u8, y: u8 },
    // ANNN, Sets I to the address NNN
    MovI { nnn: u16 },
    // BNNN, indexed jump, jump to NNN + V0, Ambiguous 
    JumpIndexed { nnn: u16 },
    // CXNN, Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN
    Rand { x: u8, nn: u8 },
    // DXYN, Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after the execution of this instruction. VF will be set if a screen pixel was changed
    Draw { x: u8, y: u8, n: u8 },
    // EX9E, Skips the next instruction if the key stored in VX is pressed
    SkipKeyEq { x: u8 },
    // EXA1, Skips the next instruction if the key stored in VX is not pressed
    SkipKeyNeq { x: u8 },
    // FX07, Sets VX to the value of the delay timer
    GetDelayTimer { x: u8 },
    // FX0A, A key press is awaited, and then stored in VX
    WaitKey { x: u8 },
    // FX15, set delay timer to VX
    SetDelayTimer { x: u8 },
    // FX18, Sets the sound timer to VX.
    SetSoundTimer { x: u8 },
    // FX1E, Adds VX to I. VF is not affected.
    AddI { x: u8 },
    // FX29, Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
    SetFontI { x: u8 },
    // FX33, Stores the binary-coded decimal representation of VX, with the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    BCD { x: u8 },
    // FX55, Stores from V0 to VX (including VX) in memory, starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
    RegDump { x: u8 },
    // FX65, Fills from V0 to VX (including VX) with values from memory, starting at address I. The offset from I is increased by 1 for each value read, but I itself is left unmodified
    RegLoad { x: u8 },
}

impl Instruction {
    pub fn decode(op_code: u16) -> Instruction {
        let nibbles = Instruction::code_to_nibble_array(op_code);

        if nibbles[0] == 0 {
            if nibbles[1] == 0 && nibbles[2] == 0xE && nibbles[3] == 0 {
                return Instruction::Cls;
            } else if nibbles[1] == 0 && nibbles[2] == 0xE && nibbles[3] == 0xE {
                return Instruction::Rts;
            } else {
                return Instruction::Invalid;
            }
        }

        if nibbles[0] == 1 {
            return Instruction::Jump {
                nnn: Instruction::combine_nibbles(&nibbles[1..]),
            };
        }

        if nibbles[0] == 2 {
            return Instruction::Call {
                nnn: Instruction::combine_nibbles(&nibbles[1..]),
            };
        }

        if nibbles[0] == 3 {
            return Instruction::SkipEqConst {
                x: nibbles[1] as u8,
                nn: Instruction::combine_nibbles(&nibbles[2..]) as u8,
            };
        }

        if nibbles[0] == 4 {
            return Instruction::SkipNeqConst {
                x: nibbles[1] as u8,
                nn: Instruction::combine_nibbles(&nibbles[2..]) as u8,
            };
        }

        if nibbles[0] == 5 {
            if nibbles[3] != 0 {
                return Instruction::Invalid;
            }

            return Instruction::SkipEq {
                x: nibbles[1] as u8,
                y: nibbles[2] as u8,
            };
        }

        if nibbles[0] == 6 {
            return Instruction::MovConst {
                x: nibbles[1] as u8,
                nn: Instruction::combine_nibbles(&nibbles[2..]) as u8,
            };
        }

        if nibbles[0] == 7 {
            return Instruction::AddConst {
                x: nibbles[1] as u8,
                nn: Instruction::combine_nibbles(&nibbles[2..]) as u8,
            };
        }

        if nibbles[0] == 8 {
            let x = nibbles[1] as u8;
            let y = nibbles[2] as u8;
            if nibbles[3] == 0 {
                return Instruction::Mov { x, y };
            }

            if nibbles[3] == 1 {
                return Instruction::Or { x, y };
            }

            if nibbles[3] == 2 {
                return Instruction::And { x, y };
            }

            if nibbles[3] == 3 {
                return Instruction::Xor { x, y };
            }

            if nibbles[3] == 4 {
                return Instruction::Add { x, y };
            }

            if nibbles[3] == 5 {
                return Instruction::SubXY { x, y };
            }

            if nibbles[3] == 6 {
                return Instruction::RightShift { x, y };
            }

            if nibbles[3] == 7 {
                return Instruction::SubYX { x, y };
            }

            if nibbles[3] == 0xE {
                return Instruction::LeftShift { x, y };
            }
        }

        if nibbles[0] == 9 {
            if nibbles[3] == 0 {
                return Instruction::SkipNeq {
                    x: nibbles[1] as u8,
                    y: nibbles[2] as u8,
                };
            }
        }

        if nibbles[0] == 0xA {
            return Instruction::MovI {
                nnn: Instruction::combine_nibbles(&nibbles[1..]),
            };
        }

        if nibbles[0] == 0xB {
            return Instruction::JumpIndexed {
                nnn: Instruction::combine_nibbles(&nibbles[1..]),
            };
        }

        if nibbles[0] == 0xC {
            return Instruction::Rand {
                x: nibbles[1] as u8,
                nn: Instruction::combine_nibbles(&nibbles[2..]) as u8,
            };
        }

        if nibbles[0] == 0xD {
            return Instruction::Draw {
                x: nibbles[1] as u8,
                y: nibbles[2] as u8,
                n: nibbles[3] as u8,
            };
        }

        if nibbles[0] == 0xE {
            let x = nibbles[1] as u8;
            if nibbles[2] == 9 && nibbles[3] == 0xE {
                return Instruction::SkipKeyEq { x };
            }

            if nibbles[2] == 0xA && nibbles[3] == 1 {
                return Instruction::SkipKeyNeq { x };
            }
        }

        if nibbles[0] == 0xF {
            let x = nibbles[1] as u8;
            if nibbles[2] == 0 && nibbles[3] == 7 {
                return Instruction::GetDelayTimer { x };
            }

            if nibbles[2] == 0 && nibbles[3] == 0xA {
                return Instruction::WaitKey { x };
            }

            if nibbles[2] == 1 && nibbles[3] == 5 {
                return Instruction::SetDelayTimer { x };
            }

            if nibbles[2] == 1 && nibbles[3] == 8 {
                return Instruction::SetSoundTimer { x };
            }

            if nibbles[2] == 1 && nibbles[3] == 0xE {
                return Instruction::AddI { x };
            }

            if nibbles[2] == 2 && nibbles[3] == 9 {
                return Instruction::SetFontI { x };
            }

            if nibbles[2] == 3 && nibbles[3] == 3 {
                return Instruction::BCD { x };
            }

            if nibbles[2] == 5 && nibbles[3] == 5 {
                return Instruction::RegDump { x };
            }

            if nibbles[2] == 6 && nibbles[3] == 5 {
                return Instruction::RegLoad { x };
            }
        }

        return Instruction::Invalid;
    }

    fn code_to_nibble_array(op_code: u16) -> [u16; 4] {
        [
            (op_code & 0xF000) >> 12,
            (op_code & 0x0F00) >> 8,
            (op_code & 0x00F0) >> 4,
            op_code & 0x000F,
        ]
    }

    fn combine_nibbles(nibbles: &[u16]) -> u16 {
        let mut combined = 0;
        for (i, nibble) in nibbles.iter().enumerate() {
            combined = combined | (*nibble << ((nibbles.len() - 1 - i) * 4));
        }
        combined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn u8_to_bool_test() {
        let byte: u8 = 0b10110011;
        let array = u8_to_bool_array(byte);
        assert_eq!(array, [true, false, true, true, false, false, true, true]);
        let byte: u8 = 0b00000000;
        let array = u8_to_bool_array(byte);
        assert_eq!(
            array,
            [false, false, false, false, false, false, false, false]
        );
        let byte: u8 = 0b11111111;
        let array = u8_to_bool_array(byte);
        assert_eq!(array, [true, true, true, true, true, true, true, true]);
    }

    
}
