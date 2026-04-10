#![no_std]
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;

pub fn tim2_us_init(periph: &stm32f411::Peripherals) {
    // enable clock access to TIM2
    periph.RCC.apb1enr.modify(|_, w| w.tim2en().set_bit());

    // page 364 of manual
    // clock is 16MHz so we downscale to 1MHz
    // periph.TIM2.psc.write(|w| w.psc().bits(16 - 1));
    periph.TIM2.psc.write(|w| w.psc().bits(16 - 1));
    periph.TIM2.arr.write(|w| w.arr().bits(0xFFFFFFFF)); // auto reload at max
    periph.TIM2.cnt.write(|w| w.cnt().bits(0)); // reset counter

    // set prescaler immediately
    // latch PSC/ARR immediately so prescaler takes effect
    periph.TIM2.egr.write(|w| w.ug().set_bit());

    // start timer
    periph.TIM2.cr1.write(|w| w.cen().set_bit());
}

pub fn micros(timer: &stm32f411::TIM2) -> u32 {
    let count = timer.cnt.read().cnt().bits();
    count
}
