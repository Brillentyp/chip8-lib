use std::{rc::Rc, cell::RefCell};

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
    fn start(&self, time:u8);
}

/// The chip8 timer is a 8-Bit timer that decrements its internal value 60 times a second. Chip8 has two timers. 
/// The sound timer should be implemented with the [Beeper] trait. This trait is intended for the delay timer.
/// 
/// For further details see: <https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#timers.>
pub trait Timer {
    fn set(&mut self, val:u8);
    fn get(&self) -> u8;
}

// going with trait objects might be a better depending on what exactly I want to do with the GUI
// I do not know wether the Rc<RefCell<_>> is really the right joice or even really necessary at this point
pub struct State<D: Display, K: Keypad, B: Beeper, T: Timer>{
    memory: Vec<u8>,
    // u16 should be enough for the usual 4k, but usize should be better for indexing the memory vector
    pc: usize,
    index_reg: u16,
    stack: Vec<u16>,
    // the 16 general purpose registers
    gp_registers: [u8; 16],

    display: Rc<RefCell<D>>,
    delay_timer: Rc<RefCell<T>>,
    sound_timer: Rc<RefCell<B>>,
    keypad: Rc<RefCell<K>>,
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
