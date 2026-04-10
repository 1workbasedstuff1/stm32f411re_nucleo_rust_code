#![no_std]
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411::{self, USART2};
// use stm32f4::stm32f411::{Interrupt, NVIC};

/// Compute the UART baud rate register value
fn compute_uart_bd(periph_clk: u32, baudrate: u32) -> u16 {
    // Equivalent to: (periph_clk + (baudrate / 2)) / baudrate
    ((periph_clk + (baudrate / 2)) / baudrate) as u16
}

/// Set the baud rate for USART2
fn uart_set_baudrate(
    usart2: &stm32f411::USART2,
    periph_clk: u32,
    baudrate: u32,
) {
    let brr_val = compute_uart_bd(periph_clk, baudrate);
    usart2.brr.write(|w| unsafe { w.bits(brr_val as u32) });
}

pub fn uart2_rx_tx_init(periph: &stm32f411::Peripherals) {
    // enable clock access to GPIOA
    periph.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());

    // PA2 and PA3 are rx tx pins for uart
    // set PA2 to alternate function mode
    periph.GPIOA.moder.modify(|_, w| w.moder2().alternate());

    // set PA3 to alternate function mode
    periph.GPIOA.moder.modify(|_, w| w.moder3().alternate());

    // set pa2 and pa3 to uart mode on alternate function
    // register
    // this is AF7 (usart2_rx)
    periph.GPIOA.afrl.modify(|_, w| w.afrl2().af7());

    // this is AF7 (usart2_tx)
    periph.GPIOA.afrl.modify(|_, w| w.afrl3().af7());

    // enable clock access to uart module
    periph.RCC.apb1enr.modify(|_, w| w.usart2en().set_bit());

    // clock is 16Mhz and we want 115200 baud rate
    uart_set_baudrate(&periph.USART2, 16_000_000, 115200);

    // select DMA for tx and rx
    // dmat: dma transmit
    // dmar: dma recieve
    // control register 3
    periph.USART2.cr3.modify(|_, w| w.dmat().set_bit());
    periph.USART2.cr3.modify(|_, w| w.dmar().set_bit());

    // set transfer direction
    periph.USART2.cr1.modify(|_, w| w.te().set_bit());
    periph.USART2.cr1.modify(|_, w| w.re().set_bit());

    // clear transmit complete flag
    periph.USART2.sr.modify(|_, w| w.tc().clear_bit());

    // enable TCIE
    // transmit complete interrupt flag
    periph.USART2.cr1.modify(|_, w| w.tcie().set_bit());

    // enable uart module
    periph.USART2.cr1.modify(|_, w| w.ue().set_bit());

    // enable USART2 interrupt in the NVIC
    unsafe {
        stm32f411::NVIC::unmask(stm32f411::Interrupt::USART2);
    }
}

pub fn dma1_init(periph: &stm32f411::Peripherals) {
    // enable clock access for DMA
    periph.RCC.ahb1enr.modify(|_, w| w.dma1en().set_bit());

    // enable DMA Stream6 Interrupt in NVIC
    unsafe {
        stm32f411::NVIC::unmask(stm32f411::Interrupt::DMA1_STREAM6);
    }
}

/// Configures DMA1 Stream6 for UART TX transfers (Memory to Peripheral)
///
/// # Arguments
/// * `periph` - Reference to the STM32F411 peripherals
/// * `msg_to_send` - Raw pointer (as u32) to the message in memory to send
/// * `msg_len` - Number of bytes to transfer
pub fn dma1_stream6_uart_tx_config(
    periph: &stm32f411::Peripherals,
    msg_to_send: u32,
    msg_len: u32,
) {
    let dma1 = &periph.DMA1;
    let usart2 = &periph.USART2;

    // Disable DMA stream before configuration
    dma1.st[6].cr.modify(|_, w| w.en().clear_bit());

    // Wait until DMA stream is fully disabled
    while dma1.st[6].cr.read().en().bit_is_set() {}

    // Clear interrupt flags for stream 6:
    // - CDMEIF6: Clear direct mode error interrupt flag
    // - CTEIF6:  Clear transfer error interrupt flag
    // - CTCIF6:  Clear transfer complete interrupt flag
    dma1.hifcr.write(|w| {
        w.cdmeif6()
            .clear_bit()
            .cteif6()
            .clear_bit()
            .ctcif6()
            .clear_bit()
    });

    // Set peripheral address to USART2 data register (DR)
    dma1.st[6]
        .par
        .write(|w| unsafe { w.pa().bits(usart2.dr.as_ptr() as u32) });

    // Set memory address to the message buffer we want to send
    dma1.st[6]
        .m0ar
        .write(|w| unsafe { w.m0a().bits(msg_to_send) });

    // Set number of bytes to transfer
    dma1.st[6]
        .ndtr
        .write(|w| unsafe { w.ndt().bits(msg_len as u16) });

    // Select Channel 4 (USART2_TX is on DMA1 Stream6 Channel4)
    // Channel bits [27:25] = 0b100
    dma1.st[6].cr.modify(|_, w| unsafe { w.chsel().bits(4) });

    // Enable memory address increment mode so DMA steps through
    // the buffer byte by byte on each transfer
    dma1.st[6].cr.modify(|_, w| w.minc().set_bit());

    // Set transfer direction: Memory to Peripheral (DIR = 0b01)
    // Data flows from our message buffer into USART2->DR
    dma1.st[6].cr.modify(|_, w| w.dir().memory_to_peripheral());

    // Enable transfer complete interrupt so we get notified
    // when the entire message has been sent
    dma1.st[6].cr.modify(|_, w| w.tcie().set_bit());

    // Enable DMA stream to begin the transfer
    dma1.st[6].cr.modify(|_, w| w.en().set_bit());
}
