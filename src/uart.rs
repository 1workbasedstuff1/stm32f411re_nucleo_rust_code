#![no_std]
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;

/// Compute the UART baud rate register value
fn compute_uart_bd(periph_clk: u32, baudrate: u32) -> u16 {
    // Equivalent to: (periph_clk + (baudrate / 2)) / baudrate
    ((periph_clk + (baudrate / 2)) / baudrate) as u16
}

/// Set the baud rate for USART2
fn uart_set_baudrate(usart2: &stm32f411::USART2, periph_clk: u32, baudrate: u32) {
    let brr_val = compute_uart_bd(periph_clk, baudrate);
    usart2.brr.write(|w| unsafe { w.bits(brr_val as u32) });
}

pub fn uart2_init(dp: &stm32f411::Peripherals) {
    // set gpioaen
    dp.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());

    // set PA2 to alternate function mode
    dp.GPIOA.moder.modify(|_, w| w.moder2().alternate());

    // set alternate function to UART2_TX
    // UART2_TX is AF07
    // afrl2 corresponds to pin2
    dp.GPIOA.afrl.modify(|_, w| w.afrl2().af7());

    // enable clock access to uart2
    dp.RCC.apb1enr.modify(|_, w| w.usart2en().set_bit());

    // stm32f411 clock rate is 16MHz
    uart_set_baudrate(&dp.USART2, 16_000_000, 115200);

    // configure transfer direction
    // WARN: may need to clear whole register to match the
    // C code
    dp.USART2.cr1.modify(|_, w| w.te().set_bit());

    // enable uart module
    dp.USART2.cr1.modify(|_, w| w.ue().set_bit());
}

pub fn uart_write(dp: &stm32f411::USART2, ch: u8) {
    // Wait until TXE (Transmit Data Register Empty) is set
    while dp.sr.read().txe().bit_is_clear() {}

    //write to transmit data register
    //this is bit mask. int is 32 bits, and
    //0xFF masks upper 24 bit

    // Write the data (lower 8 bits) to DR
    dp.dr.write(|w| unsafe { w.bits(ch as u32 & 0xFF) }); //common defensive practice
}

pub fn uart2_print(dp: &stm32f411::USART2, s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' {
            uart_write(dp, b'\r');
        }
        uart_write(dp, byte);
    }
}

pub struct Uart<'a> {
    usart: &'a stm32f411::USART2,
}

impl<'a> Uart<'a> {
    pub fn new(usart: &'a stm32f411::USART2) -> Self {
        Uart { usart }
    }
}

impl<'a> core::fmt::Write for Uart<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        uart2_print(self.usart, s);
        Ok(())
    }
}
