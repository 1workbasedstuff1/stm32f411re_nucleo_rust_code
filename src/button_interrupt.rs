#![no_std]
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411::{self, Interrupt, NVIC};

// enable interrupts for PC13 which is our input pin
pub fn pc13_exti_init(periph: &stm32f411::Peripherals) {
    // disable global interrupts
    cortex_m::interrupt::disable();

    // enable clock access for PC13
    periph.RCC.ahb1enr.modify(|_, w| w.gpiocen().set_bit());

    // set pin13 as input
    periph.GPIOC.moder.modify(|_, w| w.moder13().input());

    // enable clock access to SYSCFG
    // we do this so features like interrupts can be
    // enabled
    // allows us to connects GPIO port to EXTI line
    periph.RCC.apb2enr.modify(|_, w| w.syscfgen().set_bit());

    // select port PORTC for EXTI13
    periph
        .SYSCFG
        .exticr4
        .write(|w| unsafe { w.exti13().bits(0b0010) });

    // unmask EXTI13
    periph.EXTI.imr.modify(|_, w| w.mr13().set_bit());

    // select falling edge trigger for interrupt
    periph.EXTI.ftsr.modify(|_, w| w.tr13().set_bit());

    // Enable EXTI13 line in NVIC
    // Enable intrrupts for pins 10 to 15
    unsafe {
        NVIC::unmask(Interrupt::EXTI15_10);
    }
    unsafe {
        cortex_m::interrupt::enable();
    }
}

pub fn led_init(periph: &stm32f411::Peripherals) {
    periph.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());

    // PA5 to output
    periph.GPIOA.moder.modify(|_, w| w.moder5().output());
}

pub fn led_on(led: &stm32f411::GPIOA) {
    led.bsrr.write(|w| w.bs5().set_bit());
}

pub fn led_off(led: &stm32f411::GPIOA) {
    led.bsrr.write(|w| w.br5().set_bit());
}
