use std::{rc::Rc, cell::RefCell};

const MEM_SIZE: usize = 0xFFF + 1; // 4KiB

// it is apparently popular to put the font at 050–09F ... so I will do that as well
const FONT_START: usize = 0x50;

// for compability with older programs
const PROGRAM_START: usize = 0x200;

pub const DEFAULT_FONT: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
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
                                    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
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
    /// * 'x' - x display start position
    /// * 'y' - y display start position
    ///  
    fn modify(&mut self, sprite: &[u8], x:u8, y:u8) -> bool;
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
    fn start(&mut self, time:u8);
}

/// The chip8 timer is a 8-Bit timer that decrements its internal value 60 times a second. Chip8 has two timers. 
/// The sound timer should be implemented with the [Beeper] trait. This trait is intended for the delay timer.
/// 
/// For further details see: <https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#timers.>
pub trait Timer {
    fn set(&mut self, val:u8);
    fn get(&self) -> u8;
}

// choosing trait objects to make gui stuff easier
pub struct State{
    memory: Vec<u8>,
    // u16 should be enough for the usual 4k, but usize should be better for indexing the memory vector
    pc: usize,
    index_reg: u16,
    stack: Vec<u16>,
    // the 16 general purpose registers
    gp_registers: [u8; 16],

    display: Rc<RefCell<dyn Display>>,
    delay_timer: Rc<RefCell<dyn Timer>>,
    sound_timer: Rc<RefCell<dyn Beeper>>,
    keypad: Rc<RefCell<dyn Keypad>>,
}



// Some mock structs for testing and debugging
// ----------------------------------------------------------------
struct DebugDisplay{
    ret: bool
}
impl Display for DebugDisplay {
    fn modify(&mut self, sprite: &[u8], x:u8, y:u8) -> bool {
        self.ret
    }
}

struct DebugKepad{
    currently_pressed: Option<u8>
}
impl Keypad for DebugKepad {
    fn get_pressed_key(&self) -> Option<u8> {
        self.currently_pressed
    }
}

struct DebugBeeper{
    value: u8
}
impl Beeper for DebugBeeper {
    fn start(&mut self, time:u8) {
        self.value = time;
    }
}

struct DebugTimer{
    value: u8
}
impl Timer for DebugTimer {
    fn get(&self) -> u8 {
        self.value
    }

    fn set(&mut self, val:u8) {
        self.value = val;
    }
}

// ----------------------------------------------------------------

impl State {
    fn new(display: Rc<RefCell<dyn Display>>, delay_timer: Rc<RefCell<dyn Timer>>, sound_timer: Rc<RefCell<dyn Beeper>>, keypad: Rc<RefCell<dyn Keypad>>,) -> Self{
        State { memory: Vec::with_capacity(MEM_SIZE), pc: 0, index_reg: 0, stack: Vec::new(), gp_registers: [0; 16], display, delay_timer, sound_timer, keypad }
    }

    fn initialize(&mut self, program: &[u8], font: &[u8]){

        // load program into memory
        let program_start = 0x200;
        for i in 0..program.len(){
            self.memory[program_start + 1] = program[i];
        }
        self.pc = program_start;
        
        
        for i in 0..font.len(){
            self.memory[FONT_START + i] = font[i];
        }


        todo!();
    }

    // execute the next instruction located at pc
    fn execute() {
        todo!();
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
}
