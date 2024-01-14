use core::arch::asm;

use embedded_hal::digital::OutputPin;
use esp32c6_hal::delay;
use smart_leds::RGB8;

use embassy_time::{block_for, Duration};

pub struct Ws2812<PIN> {
    pin: PIN,
}

impl<PIN> Ws2812<PIN>
where
    PIN: OutputPin,
{
    pub fn new(mut pin: PIN) -> Ws2812<PIN> {
        pin.set_low().ok();
        Self { pin }
    }

    pub fn reset(&mut self) {
        self.pin.set_low().ok();
        block_for(Duration::from_micros(300));
    }

    fn spin_loop(count: u32) {
        let mut counter = 0;
        unsafe {
            asm!(
                "1:",
                "addi {1}, {1}, 1",
                "bne {1}, {0}, 1b",
                in (reg) count,
                inout (reg) counter,
            )
        }
    }

    /// Write a single color for ws2812 devices
    fn write_byte(&mut self, mut data: u8) {
        for _ in 0..8 {
            if (data & 0x80) != 0 {
                self.pin.set_high().ok();
                Self::spin_loop(19);
                self.pin.set_low().ok();
                Self::spin_loop(8);
            } else {
                self.pin.set_high().ok();
                Self::spin_loop(8);
                self.pin.set_low().ok();
                Self::spin_loop(19);
            }
            data <<= 1;
        }
    }

    pub fn write(&mut self, iterator: impl Iterator<Item = RGB8>) -> Result<(), ()> {
        critical_section::with(|cs| {
            for item in iterator {
                self.write_byte(item.g);
                self.write_byte(item.r);
                self.write_byte(item.b);
            }
        });

        Ok(())
    }
}
