use core::fmt::{self, Write};

pub const PL011_BASE: usize = 0x09000000;
pub const UART0_DR: *mut u32 = PL011_BASE as *mut u32;
pub const UART0_FR: *const u32 = (PL011_BASE + 0x18) as *const u32;
pub const UARTFR_TXFF: u32 = 1 << 5;

pub struct Uart;

impl Uart {
    #[inline(always)]
    pub fn write_byte(byte: u8) {
        unsafe {
            while (core::ptr::read_volatile(UART0_FR) & UARTFR_TXFF) != 0 {}
            core::ptr::write_volatile(UART0_DR, byte as u32);
        }
    }

    #[inline(always)]
    pub fn puts(s: &str) {
        for b in s.bytes() {
            if b == b'\n' {
                Self::write_byte(b'\r');
            }
            Self::write_byte(b);
        }
    }
}

#[repr(align(8))]
pub struct UartWriter;

impl Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Uart::puts(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut w = $crate::uart::UartWriter;
        let _ = write!(w, $($arg)*);
    });
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut w = $crate::uart::UartWriter;
        let _ = writeln!(w, $($arg)*);
    });
}
