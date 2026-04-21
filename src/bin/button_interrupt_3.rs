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
use stm32f411_nucleo::uart::uart2_print;

struct SharedPeripherals {
    uart2: stm32f411::USART2,
    exti: stm32f411::EXTI,
}

// put all the peripherals we'll use into one struct
static PERIPHERALS: Mutex<RefCell<Option<SharedPeripherals>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let periph = stm32f411::Peripherals::take().unwrap();
    uart::uart2_init(&periph, 115200);
    button_interrupt::led_init(&periph);
    button_interrupt::pc13_exti_init(&periph);

    // Move USART2 into the global static
    // interrupt free disables interrupts, runs closure then
    // re-enables interrupts
    // critical section is where interrupts are disabled
    cortex_m::interrupt::free(|cs| {
        // modify USART2
        // USART2.borrow(cs).replace(Some(periph.USART2));
        PERIPHERALS.borrow(cs).replace(Some(SharedPeripherals {
            uart2: periph.USART2,
            exti: periph.EXTI,
        }))
    });

    // uart::uart2_print(&periph.USART2, "begin");

    loop {}
}

#[interrupt]
fn EXTI15_10() {
    // in interrupt free section, check what bit caused the interrupt
    cortex_m::interrupt::free(|cs| {
        if let Some(shared) = PERIPHERALS.borrow(cs).borrow_mut().as_mut() {
            if shared.exti.pr.read().pr13().bit_is_set() {
                // write 1 to clear
                shared.exti.pr.write(|w| w.pr13().set_bit());

                // let mut uart = uart::Uart::new(&mut shared.usart);
                // write!(uart, "button was pressed\r\n").ok();
                // NOTE: avoid the write function
                uart::uart2_print(&shared.uart2, "button was pressed\r\n");
            }
        }
    });
}
