#![no_std]
use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;

pub fn fpu_init(periph: &stm32f411::Peripherals) {
    // this is a 4 bit register, want to set it to 1111
    // which is equal to 15 or 0xF
    periph
        .FPU_CPACR
        .cpacr
        .write(|w| unsafe { w.cp().bits(0xF) });

    cortex_m::asm::dsb(); // Data synchronization
    cortex_m::asm::isb(); // Instruction synchronization
}

pub fn fpu_disable(periph: &stm32f411::Peripherals) {
    periph.FPU_CPACR.cpacr.write(|w| unsafe { w.cp().bits(0) });
}

// // #[inline]
// pub fn fast_sqrt(x: f32) -> f32 {
//     let result: f32;
//     unsafe {
//         core::arch::asm!(
//             "VSQRT.F32 {0}, {1}",
//             out(sreg) result,
//             in(sreg) x,
//         );
//     }
//     result
// }
//
#[inline(never)]
pub fn fpu_multiply_accumulate(acc: f32, a: f32, b: f32) -> f32 {
    let result: f32;
    unsafe {
        core::arch::asm!(
            "vmul.f32 {tmp}, {a}, {b}",
            "vadd.f32 {out}, {acc}, {tmp}",
            a = in(sreg) a,
            b = in(sreg) b,
            acc = in(sreg) acc,
            tmp = out(sreg) _,
            out = out(sreg) result,
        );
    }
    result
}
