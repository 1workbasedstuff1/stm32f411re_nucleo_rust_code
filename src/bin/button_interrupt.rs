#![no_std]
#![no_main]

use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;
use stm32f411::interrupt;
use stm32f411_nucleo::button_interrupt;
use stm32f411_nucleo::uart;

// Global static to hold USART2 peripheral, shared between main and interrupt
// use mutex to prevent interrupt races
// Mutex disables interrupts for the duration of the hold
// refcell to mutate a shared reference
static USART2: Mutex<RefCell<Option<stm32f411::USART2>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let periph = stm32f411::Peripherals::take().unwrap();
    uart::uart2_init(&periph);
    button_interrupt::led_init(&periph);
    button_interrupt::pc13_exti_init(&periph);

    // Move USART2 into the global static
    // interrupt free disables interrupts, runs closure then
    // re-enables interrupts
    // critical section is where interrupts are disabled
    cortex_m::interrupt::free(|cs| {
        // modify USART2
        USART2.borrow(cs).replace(Some(periph.USART2));
    });

    loop {}
}

#[interrupt]
fn EXTI15_10() {
    let dp = unsafe { stm32f411::Peripherals::steal() };

    if dp.EXTI.pr.read().pr13().bit_is_set() {
        // write 1 to clear the register
        dp.EXTI.pr.write(|w| w.pr13().set_bit());

        // Borrow USART2 from the global static
        cortex_m::interrupt::free(|cs| {
            if let Some(usart) = USART2.borrow(cs).borrow_mut().as_mut() {
                let mut uart = uart::Uart::new(usart);
                write!(uart, "button was pressed\n").ok();
            }
        });
    }
}
