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
    fn modify(&self, sprite: &[u8], x:u8, y:u8) -> bool;
}

///
/// The chip8 keypad is hexdecimal. It contains buttons for 0-9 and A-F.
/// 
/// For further details see: <https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#keypad>
pub trait Keypad {
    ///
    /// Returns `Some<_>` if a key is *currently* pressed, `None` otherwise. The Some contains the pressed key as an `u8` (0x0 .. 0xF)
    fn get_pressed_key() -> Option<u8>;
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
    fn set(val:u8);
    fn get() -> u8;
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
