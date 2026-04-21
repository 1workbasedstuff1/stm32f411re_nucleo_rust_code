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

/// Set the baud rate for USART2 (APB1)
fn uart_set_baudrate(
    usart2: &stm32f411::USART2,
    periph_clk: u32,
    baudrate: u32,
) {
    let brr_val = compute_uart_bd(periph_clk, baudrate);
    usart2.brr.write(|w| unsafe { w.bits(brr_val as u32) });
}

pub fn uart2_init(dp: &stm32f411::Peripherals, baudrate: u32) {
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
    uart_set_baudrate(&dp.USART2, 16_000_000, baudrate);

    // configure transfer direction
    // WARN: may need to clear whole register to match the
    // C code
    dp.USART2.cr1.modify(|_, w| w.te().set_bit());

    // enable uart module
    dp.USART2.cr1.modify(|_, w| w.ue().set_bit());
}

// configure PA3 for UART2_RX
// WARN: doesnt work because rx is locked on stm32 nucleo board
pub fn uart2_rx_init(dp: &stm32f411::Peripherals, baudrate: u32) {
    // set gpioaen
    dp.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());

    // set PA3 to alternative
    dp.GPIOA.moder.modify(|_, w| w.moder3().alternate());

    // set alternative function to UART2_RX
    dp.GPIOA.afrl.modify(|_, w| w.afrl3().af7());

    // enable clock access for uart2
    dp.RCC.apb1enr.modify(|_, w| w.usart2en().set_bit());

    // stm32f411 clock rate is 16MHz
    uart_set_baudrate(&dp.USART2, 16_000_000, baudrate);

    // Enable RX (receive) ⭐ important for GPS
    dp.USART2.cr1.modify(|_, w| w.re().set_bit());

    // Enable USART peripheral
    dp.USART2.cr1.modify(|_, w| w.ue().set_bit());
}

pub fn uart2_rx_tx_init(dp: &stm32f411::Peripherals, baudrate: u32) {
    // 1. Enable GPIOA clock
    dp.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());

    // 2. Set PA2 (TX) + PA3 (RX) to Alternate Function mode
    dp.GPIOA
        .moder
        .modify(|_, w| w.moder2().alternate().moder3().alternate());

    // 3. Set AF7 for PA2 and PA3 (USART2)
    // AFRL controls pins 0-7
    dp.GPIOA.afrl.modify(|_, w| w.afrl2().af7().afrl3().af7());

    // 4. Enable USART2 clock
    dp.RCC.apb1enr.modify(|_, w| w.usart2en().set_bit());

    // 5. Set baud rate (16 MHz clock assumption)
    uart_set_baudrate(&dp.USART2, 16_000_000, baudrate);

    // 6. Enable TX + RX
    dp.USART2.cr1.modify(|_, w| {
        w.te()
            .set_bit() // transmitter
            .re()
            .set_bit() // receiver ⭐ IMPORTANT
    });

    // 7. Enable USART
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

// uart rx pin reading
// pub fn uart_read(usart2: &stm32f411::USART2) -> u8 {
//     // Wait until RXNE (Receive Not Empty)
//     while usart2.sr.read().rxne().bit_is_clear() {}
//
//     // Read received byte
//     usart2.dr.read().dr().bits() as u8
// }

pub fn uart_read(usart2: &stm32f411::USART2) -> Option<u8> {
    if usart2.sr.read().rxne().bit_is_set() {
        Some(usart2.dr.read().dr().bits() as u8)
    } else {
        None
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

/// Set the baud rate for USART1 (APB2)
fn uart1_set_baudrate(
    usart1: &stm32f411::USART1,
    periph_clk: u32,
    baudrate: u32,
) {
    let brr_val = compute_uart_bd(periph_clk, baudrate);
    usart1.brr.write(|w| unsafe { w.bits(brr_val as u32) });
}

/// USART1 on PA9 (TX) / PA10 (RX), AF7 — use for peripherals (e.g. GPS).
/// Nucleo-F411RE routes USART2 (PA2/PA3) to ST-Link; GPS TX must go to a **different** MCU RX.
pub fn uart1_rx_tx_init(dp: &stm32f411::Peripherals, baudrate: u32) {
    dp.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());

    // set PA9 and PA10 to alternate
    dp.GPIOA
        .moder
        .modify(|_, w| w.moder9().alternate().moder10().alternate());

    // set both to af7 for uart functionality
    dp.GPIOA.afrh.modify(|_, w| w.afrh9().af7().afrh10().af7());

    // enable clock access
    dp.RCC.apb2enr.modify(|_, w| w.usart1en().set_bit());

    // Default reset clock: HSI 16 MHz, APB2 prescaler 1 → USART1 kernel clock 16 MHz
    uart1_set_baudrate(&dp.USART1, 16_000_000, baudrate);

    dp.USART1.cr1.modify(|_, w| w.te().set_bit().re().set_bit());
    dp.USART1.cr1.modify(|_, w| w.ue().set_bit());
}

pub fn uart1_read(usart1: &stm32f411::USART1) -> u8 {
    while usart1.sr.read().rxne().bit_is_clear() {}
    usart1.dr.read().dr().bits() as u8
}

// NOTE: ore is overrun error which causes GPS to stop working
// adding this in means it works continually
pub fn uart1_try_read(usart1: &stm32f411::USART1) -> Option<u8> {
    let sr = usart1.sr.read();
    // Handle UART errors FIRST
    if sr.ore().bit_is_set() || sr.fe().bit_is_set() || sr.nf().bit_is_set() {
        // Clear error by reading DR
        let _ = usart1.dr.read().dr().bits();
        return None;
    }
    if usart1.sr.read().rxne().bit_is_set() {
        Some(usart1.dr.read().dr().bits() as u8)
    } else {
        None
    }
}
