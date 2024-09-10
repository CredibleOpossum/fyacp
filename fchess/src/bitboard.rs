use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Mul, Not};

use colored::Colorize;

pub static UNIVERSE: u64 = u64::MAX;

#[derive(Clone, Copy)]
pub struct BitBoard(pub u64);

impl BitBoard {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn set_bit(&mut self, bit_index: u8) {
        self.0 |= 1 << bit_index;
    }
    #[inline]
    pub fn clear_bit(&mut self, bit_index: u8) {
        self.0 &= UNIVERSE ^ (1 << bit_index);
    }
    #[inline]
    pub fn get_bit(&self, bit_index: u8) -> bool {
        self.0 & (1 << bit_index) != 0
    }

    fn print_internal(&self, highlighted_position: Option<u8>) {
        for bit in (0..64).rev() {
            // This is horrifying, probably should rework.
            let should_be_highlighted = if let Some(position) = highlighted_position {
                position == bit
            } else {
                false
            };

            let bit_value = self.get_bit(bit);
            if bit_value {
                print!("{} ", (bit_value as i32).to_string().green());
            } else if should_be_highlighted {
                print!("{} ", (bit_value as i32).to_string().yellow());
            } else {
                print!("{} ", (bit_value as i32).to_string().red());
            }

            if bit % 8 == 0 {
                println!();
            }
        }
        println!("{}", self.0);
    }

    pub fn print(&self) {
        self.print_internal(None);
    }

    pub fn print_highlighting(&self, position: u8) {
        self.print_internal(Some(position));
    }
}

impl BitAnd for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }
}

impl BitAndAssign for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, other: BitBoard) {
        *self = *self & other;
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }
}

impl BitOrAssign for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, other: BitBoard) {
        *self = *self | other;
    }
}

impl Mul for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(other.0))
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> BitBoard {
        BitBoard(!self.0)
    }
}
