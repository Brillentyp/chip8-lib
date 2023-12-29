use std::sync::{Arc, Mutex};

const MEM_SIZE: usize = 0xFFF + 1; // 4KiB

// it is apparently popular to put the font at 050–09F ... so I will do that as well
const FONT_START: usize = 0x50;

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
    stack: Vec<u16>,
    // the 16 general purpose registers
    gp_registers: [u8; 16],

    display: Arc<Mutex<dyn Display>>,
    delay_timer: Arc<Mutex<dyn Timer>>,
    sound_timer: Arc<Mutex<dyn Beeper>>,
    keypad: Arc<Mutex<dyn Keypad>>,
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

            if (actual_y + line) as usize >= self.display_height {
                // sprite should clip so we are finished
                return result_flag;
            }
            
            for (i, b) in line_bools.iter().enumerate() {
                // drawing should clip
                if actual_x as usize + i < self.display_width {
                    let index = actual_x as usize + i + self.display_width * (line + actual_y) as usize;

                    // note that != is the same as a logical XOR
                    self.display[index] = self.display[index] != *b;

                    // if the bit was set a pixel was flipped
                    if *b {
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
        display: Arc<Mutex<dyn Display>>,
        delay_timer: Arc<Mutex<dyn Timer>>,
        sound_timer: Arc<Mutex<dyn Beeper>>,
        keypad: Arc<Mutex<dyn Keypad>>,
    ) -> Self {
        State {
            memory: Vec::with_capacity(MEM_SIZE),
            pc: 0,
            index_reg: 0,
            stack: Vec::new(),
            gp_registers: [0; 16],
            display,
            delay_timer,
            sound_timer,
            keypad,
        }
    }

    pub fn initialize(&mut self, program: &[u8], font: &[u8]) {
        // load program into memory
        for i in 0..program.len() {
            self.memory[PROGRAM_START + 1] = program[i];
        }
        self.pc = PROGRAM_START;

        for i in 0..font.len() {
            self.memory[FONT_START + i] = font[i];
        }
    }

    // execute the next instruction located at pc
    pub fn execute(&mut self) {
        // fetch
        let upper = self.memory[self.pc];
        let lower = self.memory[self.pc + 1];

        // keep in mind that the pc is incremented here, important for some instructions
        self.pc += 2;

        // Decode
        // split the opcode into the relevant parts

        let op_code = (upper & 0xF0) >> 4;
        // X: second nibble of instruction. Used to look up one of the 16 registers
        // Y: third nibble of instruction. Used to look up one of the 16 registers
        // N: The *fourth* nibble
        // NN: second byte, immediate 8-bit number
        // NNN: second, third and fourth nibble, immediate 12-bit address
        let x = upper & 0x0F;
        let y = (lower & 0xF0) >> 4;
        let n = lower & 0x0F;
        let nn = lower;
        let nnn: usize = 0 | (lower as usize) | ((x as usize) << 8);

        // ... tomorrow
        match op_code {
            // this instructino does not need to be implemented
            0x0 => {}
            0x1 => {}
            0x2 => {}
            0x3 => {}
            0x4 => {}
            0x5 => {}
            0x6 => {}
            0x7 => {}
            0x8 => {}
            0x9 => {}
            0xa => {}
            0xb => {}
            0xc => {}
            0xd => {}
            0xe => {}
            0xf => {}
            _ => panic!("FUUUUUUCK (unknown instruction)"),
        }
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
        assert_eq!(array, [false, false, false, false, false, false, false, false]);
        let byte: u8 = 0b11111111;
        let array = u8_to_bool_array(byte);
        assert_eq!(array, [true, true, true, true, true, true, true, true]);
    }
}

// Mnemonics are (mostly) taken from: http://www.emulator101.com/chip-8-instruction-set.html
// also https://en.wikipedia.org/wiki/CHIP-8
enum Instruction{
    // 0NNN, Instruction 0NNN calls a machine code routine (RCA 1802 for COSMAC VIP), I won't implement this instruction
    // use Invalid for this Instruction
    Invalid,
    // 00E0, clear screen
    Cls,
    // 00EE, return from subroutine
    Rts,
    // 1NNN, absolute jump to NNN
    Jump{nnn: u16},
    // 2NNN, jump to subroutine at NNN (push address to stack, change pc)
    Call{nnn: u16},
    // 3XNN, skip next instruction if Vx equals NN
    SkipEqConst{x:u8, nn:u8},
    // 4XNN, skip next instruction if Vx does not equal NN
    SkipNeqConst{x:u8, nn:u8},
    // 5XY0, skips the next instruction if VX equals VY
    SkipEq{x:u8, y:u8},
    // 6XNN, Sets VX to NN. 
    MovConst{x:u8, nn:u8},
    // 7XNN, Adds NN to VX (carry flag is not changed)
    AddConst{x:u8, nn:u8},
    // 8XY0, Sets VX to the value of VY. 
    Mov{x:u8, y:u8},
    // 8XY1, Sets VX to VX or VY. (bitwise OR operation) 
    Or{x:u8, y:u8},
    // 8XY2, Sets VX to VX and VY. (bitwise AND operation) 
    And{x:u8, y:u8},
    // 8XY3, Sets VX to VX xor VY
    Xor{x:u8, y:u8},
    // 8XY4, Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there is not. 
    Add{x:u8, y:u8},
    // 8XY5, VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there is not. 
    SubFrom{x:u8, y:u8},
    // 8XY6, Stores the least significant bit of VX in VF and then shifts VX to the right by 1 (ambiguous see chip8 guide)
    RightShift{x:u8, y:u8},
    // 8XY7, Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not. 
    Sub{x:u8, y:u8},
    // 8XYE, Stores the most significant bit of VX in VF and then shifts VX to the left by 1
    LeftShift{x:u8, y:u8},
    // 9XY0, Skips the next instruction if VX does not equal VY
    SkipNeq{x:u8, y:u8},
    // ANNN, Sets I to the address NNN
    MovI{nnn:u16},
    // BNNN, indexed jump, jump to NNN + V0
    JumpIndexed{nnn: u16},
    // CXNN, Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN
    Rand{x:u8, nn:u8},
    // DXYN, Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after the execution of this instruction. VF will be set if a screen pixel was changed
    DRAW{x:u8, y:u8, n:u8},
    // EX9E, Skips the next instruction if the key stored in VX is pressed
    SkipKeyEq{x:u8},
    // EXA1, Skips the next instruction if the key stored in VX is not pressed
    SkipKeyNeq{x:u8},
    // FX07, Sets VX to the value of the delay timer
    GetDelayTimer{x:u8},
    // FX0A, A key press is awaited, and then stored in VX
    WaitKey{x:u8},
    // FX15, set delay timer to VX
    SetDelayTimer{x:u8},
    // FX18, Sets the sound timer to VX. 
    SetSoundTimer{x:u8},
    // FX1E, Adds VX to I. VF is not affected.
    AddI{x:u8},
    // FX29, Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
    SetFontI{x:u8},
    // FX33, Stores the binary-coded decimal representation of VX, with the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2. 
    BCD{x:u8},
    // FX55, Stores from V0 to VX (including VX) in memory, starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
    RegDump{x:u8},
    // FX65, Fills from V0 to VX (including VX) with values from memory, starting at address I. The offset from I is increased by 1 for each value read, but I itself is left unmodified
    RegLoad{x:u8},
}
