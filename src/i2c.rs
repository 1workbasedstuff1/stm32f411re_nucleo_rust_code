#![no_std]
use crate::{i2c, uart};
use core::fmt::Write;
use core::ptr::read_volatile;
use cortex_m::{peripheral, register::control::write};
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f411;

// PB8 --- SCL
// PB9 --- SDA

pub fn i2c1_init(periph: &stm32f411::Peripherals) {
    // to enable B pins must enable GPIO B
    periph.RCC.ahb1enr.modify(|_, w| w.gpioben().set_bit());

    // set both pins to alternate function mode
    // PB8 set to alternate
    periph.GPIOB.moder.modify(|_, w| w.moder8().alternate());

    // PB9 set to alternate
    periph.GPIOB.moder.modify(|_, w| w.moder9().alternate());

    // Set PB8 & PB9 output type to open drain for I2C
    periph.GPIOB.otyper.modify(|_, w| w.ot8().set_bit());
    periph.GPIOB.otyper.modify(|_, w| w.ot9().set_bit());

    // Enable pull up resistors for PB8 and PB9
    periph.GPIOB.pupdr.modify(|_, w| w.pupdr8().pull_up());
    periph.GPIOB.pupdr.modify(|_, w| w.pupdr9().pull_up());

    // Set PB8 and PB9 to alternate functoin type I2C (AF4)
    // RMO383 page 151
    periph.GPIOB.afrh.modify(|_, w| w.afrh8().af4());
    periph.GPIOB.afrh.modify(|_, w| w.afrh9().af4());

    // Enable clock access to I2C
    periph.RCC.apb1enr.modify(|_, w| w.i2c1en().set_bit());

    // enter reset mode
    periph.I2C1.cr1.modify(|_, w| w.swrst().set_bit());

    // exit reset mode
    periph.I2C1.cr1.modify(|_, w| w.swrst().clear_bit());

    // set peripheral clock frequency
    // set bit 4 to 1
    periph
        .I2C1
        .cr2
        .modify(|_, w| unsafe { w.freq().bits(0b010000 as u8) });
    // .modify(|_,w| w.;

    // Set I2C to standard mode 100KHz clock
    periph.I2C1.ccr.modify(|_, w| unsafe { w.bits(80 as u32) });

    // set rise time
    periph
        .I2C1
        .trise
        .modify(|_, w| unsafe { w.bits(17 as u32) });

    periph.I2C1.cr1.modify(|_, w| w.pe().set_bit());
}

pub fn i2c1_byte_read(
    i2c1: &stm32f411::I2C1,
    saddr: u8,
    maddr: u8,
    data: &mut [u8],
) {
    let tmp: u32;

    // Wait till not busy
    while i2c1.sr2.read().busy().bit_is_set() {}

    // generate start
    i2c1.cr1.modify(|_, w| w.start().set_bit());

    // wait till start is set
    while i2c1.sr1.read().sb().bit_is_clear() {}

    // i2c1.dr.modify(|w, _| unsafe { w.bits(saddr << 1) as u32 });
    i2c1.dr.write(|w| w.dr().bits(saddr << 1));

    while !(i2c1.sr1.read().addr().bit_is_set()) {}

    // volatile read to clear status register
    let mut _tmp = i2c1.sr2.read().bits();

    // send memory address
    i2c1.dr.write(|w| w.dr().bits(maddr));

    // wait until transmitter is empty
    while !(i2c1.sr1.read().tx_e().bit_is_set()) {}

    // generate restart
    i2c1.cr1.modify(|_, w| w.start().set_bit());

    // wait till start flag is set
    while i2c1.sr1.read().sb().bit_is_clear() {}

    // transmit slave address + read
    i2c1.dr.write(|w| w.dr().bits(saddr << 1 | 1));

    // wait until addr flag is set
    while !(i2c1.sr1.read().addr().bit_is_set()) {}

    // clear acknowledge bit / disable acknowledge bit
    i2c1.cr1.modify(|_, w| w.ack().clear_bit());

    // read to clear addr flag
    // WARN: needs to be full volatile read
    // _ = i2c1.sr2.read().bits();
    let _ = unsafe { core::ptr::read_volatile(i2c1.sr2.as_ptr()) };

    // generate stop after recieved
    i2c1.cr1.modify(|_, w| w.stop().set_bit());

    // wait until RXNE flag is set
    while !(i2c1.sr1.read().rx_ne().bit_is_set()) {}

    // read data from dr
    data[0] = i2c1.dr.read().bits() as u8;
}

pub fn i2c1_burst_write(
    i2c1: &stm32f411::I2C1,
    saddr: u8,
    maddr: u8,
    n: usize,
    data: &[u8],
) {
    // Wait until bus not busy
    while i2c1.sr2.read().busy().bit_is_set() {}

    // Generate start
    i2c1.cr1.modify(|_, w| w.start().set_bit());

    // Wait until start flag is set
    while i2c1.sr1.read().sb().bit_is_clear() {}

    // Transmit slave address (shifted left by 1 for write)
    // WARN: data register is only 8 bits
    // i2c1.dr.write(|w| unsafe { w.bits((saddr << 1) as u32) });
    i2c1.dr.write(|w| w.dr().bits((saddr << 1)));

    // Wait until addr flag is set
    while i2c1.sr1.read().addr().bit_is_clear() {}

    // Clear addr flag by reading SR2
    // WARN: need to be full volatile read
    let _ = i2c1.sr2.read().bits();

    // Wait until data register empty
    while i2c1.sr1.read().tx_e().bit_is_clear() {}

    // Send memory address
    // i2c1.dr.write(|w| unsafe { w.bits(maddr as u32) });
    i2c1.dr.write(|w| w.dr().bits(maddr));

    for i in 0..n {
        // Wait until data register empty
        while i2c1.sr1.read().tx_e().bit_is_clear() {}

        // Transmit byte
        // i2c1.dr.write(|w| unsafe { w.bits(data[i] as u32) });
        i2c1.dr.write(|w| w.dr().bits(data[i]));
    }

    // Wait until transfer finished (BTF)
    while i2c1.sr1.read().btf().bit_is_clear() {}

    // Generate stop
    i2c1.cr1.modify(|_, w| w.stop().set_bit());
}

pub fn i2c1_burst_write_dbg(
    i2c1: &stm32f411::I2C1,
    saddr: u8,
    maddr: u8,
    n: usize,
    data: &[u8],
    uart: &mut uart::Uart,
) {
    write!(uart, "0: entering burst write\n").ok();
    // Wait until bus not busy
    while i2c1.sr2.read().busy().bit_is_set() {}
    write!(uart, "1: finished checking busy\n");

    // i2c1.sr1.write(|w| w.af().clear_bit());

    // Generate start
    i2c1.cr1.modify(|_, w| w.start().set_bit());
    write!(uart, "2: set start bit\n").ok();

    // Wait until start flag is set
    while i2c1.sr1.read().sb().bit_is_clear() {}
    write!(uart, "3: sb bit is clear\n").ok();

    // Transmit slave address (shifted left by 1 for write)
    // WARN: data register is only 8 bits
    // i2c1.dr.write(|w| unsafe { w.bits((saddr << 1) as u32) });
    i2c1.dr.write(|w| w.dr().bits((saddr << 1)));
    let sr1 = i2c1.sr1.read().bits();
    write!(
        uart,
        "4: data register write complete, SR1 = 0x{:04x}\n",
        sr1
    )
    .ok();
    write!(uart, "4: data register write complete\n").ok();

    while i2c1.sr1.read().addr().bit_is_clear() {}
    write!(uart, "5: addr flag is set\n").ok();

    // Clear addr flag by reading SR2
    // WARN: need to be full volatile read
    let _ = unsafe { core::ptr::read_volatile(i2c1.sr2.as_ptr()) };
    // let _ = i2c1.sr2.read().bits();
    write!(uart, "6: sr2 bits are read\n").ok();

    // Wait until data register empty
    while i2c1.sr1.read().tx_e().bit_is_clear() {}
    write!(uart, "7: txe register is empty\n").ok();

    // Send memory address
    // i2c1.dr.write(|w| unsafe { w.bits(maddr as u32) });
    i2c1.dr.write(|w| w.dr().bits(maddr));
    write!(uart, "8: write to memory address\n").ok();

    for i in 0..n {
        // Wait until data register empty
        while i2c1.sr1.read().tx_e().bit_is_clear() {}
        write!(uart, "9: txe is set\n").ok();

        // Transmit byte
        // i2c1.dr.write(|w| unsafe { w.bits(data[i] as u32) });
        i2c1.dr.write(|w| w.dr().bits(data[i]));
        write!(uart, "10: bits written to data register\n").ok();
    }

    // Wait until transfer finished (BTF)
    while i2c1.sr1.read().btf().bit_is_clear() {}
    write!(uart, "11: btf is set\n").ok();

    // Generate stop
    i2c1.cr1.modify(|_, w| w.stop().set_bit());
    write!(uart, "12: stop\n").ok();
}

pub fn i2c1_byte_read_dbg(
    i2c1: &stm32f411::I2C1,
    saddr: u8,
    maddr: u8,
    data: &mut [u8],
    uart: &mut uart::Uart,
) {
    write!(uart, "0: entering byte read\n").ok();
    let tmp: u32;

    // Wait till not busy
    while i2c1.sr2.read().busy().bit_is_set() {}
    write!(uart, "1: not busy\n").ok();

    // generate start
    i2c1.cr1.modify(|_, w| w.start().set_bit());
    write!(uart, "2: start bit set\n").ok();

    // wait till start is set
    while i2c1.sr1.read().sb().bit_is_clear() {}
    write!(uart, "3: start bit complete\n").ok();

    // i2c1.dr.modify(|w, _| unsafe { w.bits(saddr << 1) as u32 });
    i2c1.dr.write(|w| w.dr().bits(saddr << 1));
    write!(uart, "4: set dr slave address\n").ok();

    while !(i2c1.sr1.read().addr().bit_is_set()) {}
    write!(uart, "5: add bit clear\n").ok();

    // volatile read to clear status register
    let mut _tmp = i2c1.sr2.read().bits();
    write!(uart, "6: clear sr2\n").ok();

    // send memory address
    i2c1.dr.write(|w| w.dr().bits(maddr));
    write!(uart, "7: send memory address\n").ok();

    // wait until transmitter is empty
    while !(i2c1.sr1.read().tx_e().bit_is_set()) {}
    write!(uart, "8: sr1 transmitter empty\n").ok();

    // generate restart
    i2c1.cr1.modify(|_, w| w.start().set_bit());
    write!(uart, "9: generate restart\n").ok();

    // wait till start flag is set
    while i2c1.sr1.read().sb().bit_is_clear() {}
    write!(uart, "10: wait until start flag is set\n").ok();

    // transmit slave address + read
    i2c1.dr.write(|w| w.dr().bits(saddr << 1 | 1));
    write!(uart, "11: write read bytes\n").ok();

    // wait until addr flag is set
    while !(i2c1.sr1.read().addr().bit_is_set()) {}
    write!(uart, "12: sddr flag is set\n").ok();

    // clear acknowledge bit / disable acknowledge bit
    i2c1.cr1.modify(|_, w| w.ack().clear_bit());
    write!(uart, "13: disabel ack bit\n").ok();

    // read to clear addr flag
    // WARN: needs to be full volatile read
    // _ = i2c1.sr2.read().bits();
    let _ = unsafe { core::ptr::read_volatile(i2c1.sr2.as_ptr()) };
    write!(uart, "14: read to clear sr2\n").ok();

    // generate stop after recieved
    i2c1.cr1.modify(|_, w| w.stop().set_bit());
    write!(uart, "15: generate stop\n").ok();

    // wait until RXNE flag is set
    while !(i2c1.sr1.read().rx_ne().bit_is_set()) {}
    write!(uart, "16: rxne flag is set").ok();

    // read data from dr
    data[0] = i2c1.dr.read().bits() as u8;
    write!(uart, "cmplt: read from dr").ok();
}

pub fn i2c1_burst_read(
    i2c1: &stm32f411::I2C1,
    saddr: u8,
    maddr: u8,
    data: &mut [u8],
) {
    // wait until not busy
    while i2c1.sr2.read().busy().bit_is_set() {}

    // set i2c start
    i2c1.cr1.modify(|_, w| w.start().set_bit());

    // wait until start is set
    while i2c1.sr1.read().sb().bit_is_clear() {}

    // transmit slave address + write
    i2c1.dr.write(|w| w.dr().bits(saddr << 1));

    // wait until addr flag is set
    while i2c1.sr1.read().addr().bit_is_clear() {}

    // clear addr flag by reading SR2
    let _tmp = i2c1.sr2.read().bits();

    // wait until transmitter empty
    while i2c1.sr1.read().tx_e().bit_is_clear() {}

    // send memory address
    i2c1.dr.write(|w| w.dr().bits(maddr));

    // wait until transmitter empty
    while i2c1.sr1.read().tx_e().bit_is_clear() {}

    // generate restart
    i2c1.cr1.modify(|_, w| w.start().set_bit());

    // wait until start flag is set
    while i2c1.sr1.read().sb().bit_is_clear() {}

    // transmit slave address + read
    i2c1.dr.write(|w| w.dr().bits((saddr << 1) | 1));

    // wait until addr flag is set
    while i2c1.sr1.read().addr().bit_is_clear() {}

    // clear addr flag by reading SR2
    let _tmp = i2c1.sr2.read().bits();

    // enable acknowledge
    i2c1.cr1.modify(|_, w| w.ack().set_bit());

    let n = data.len();
    for i in 0..n {
        if i == n - 1 {
            // disable acknowledge
            i2c1.cr1.modify(|_, w| w.ack().clear_bit());

            // generate stop
            i2c1.cr1.modify(|_, w| w.stop().set_bit());

            // wait for RXNE flag set
            while i2c1.sr1.read().rx_ne().bit_is_clear() {}

            // read data from DR
            data[i] = i2c1.dr.read().dr().bits();
        } else {
            // wait until RXNE flag is set
            while i2c1.sr1.read().rx_ne().bit_is_clear() {}

            // read data from DR
            data[i] = i2c1.dr.read().dr().bits();
        }
    }
}
