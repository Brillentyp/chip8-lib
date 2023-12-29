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
