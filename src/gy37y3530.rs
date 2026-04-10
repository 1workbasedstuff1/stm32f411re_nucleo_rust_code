#![no_main]
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411::{self, gpioa};

// set PA2 and PA3 as GPIOA inputs

pub fn motor_setup(periph: &stm32f411::Peripherals) {
    // Enable clock access to GPIOA
    periph.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());

    // Small delay for clock stabilization
    for _ in 0..10 {
        cortex_m::asm::nop();
    }

    // Configure as inputs
    periph.GPIOA.moder.modify(|_, w| w.moder6().input());
    periph.GPIOA.moder.modify(|_, w| w.moder7().input());

    // Set pull-down on both pins
    periph.GPIOA.pupdr.modify(|_, w| {
        w.pupdr6()
            .pull_down() // 0b10 for pull-down
            .pupdr7()
            .pull_down()
    });
}

// read whether PA2 or PA3 is high
pub fn motor_read(periph: &stm32f411::GPIOA) -> (bool, bool) {
    let pa2_high = periph.idr.read().idr6().bit_is_set();
    let pa3_high = periph.idr.read().idr7().bit_is_set();

    (pa2_high, pa3_high)
}

pub struct Encoder {
    prev: (bool, bool),
    count: i32,
    counts_per_rev: i32, // e.g., 2096 for your motor+gearbox
}

impl Encoder {
    pub fn new(initial_state: (bool, bool), counts_per_rev: i32) -> Self {
        Self {
            prev: initial_state,
            count: 0,
            counts_per_rev,
        }
    }

    pub fn update(&mut self, current: (bool, bool)) {
        let (prev_a, prev_b) = self.prev;
        let (curr_a, curr_b) = current;

        match (prev_a, prev_b, curr_a, curr_b) {
            (false, false, false, true)
            | (false, true, true, true)
            | (true, true, true, false)
            | (true, false, false, false) => self.count += 1, // forward

            (false, false, true, false)
            | (true, false, true, true)
            | (true, true, false, true)
            | (false, true, false, false) => self.count -= 1, // backward

            _ => {} // no change or invalid transition
        }

        self.prev = current;
    }

    pub fn ticks(&self) -> i32 {
        // Return the current count as a positive number
        // If count can be negative, you might want to handle that
        self.count
    }
}
