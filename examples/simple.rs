#![no_main]
#![no_std]

use cs43l22::{Config, CS43L22};
use defmt_rtt as _;
use panic_probe as _;
use stm32f4xx_hal::{pac, prelude::*};

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.MHz()).sysclk(96.MHz()).freeze();
    let gpiob = dp.GPIOB.split();

    let pins = (gpiob.pb6, gpiob.pb9);
    let i2c = dp.I2C1.i2c(pins, 25.kHz(), &clocks);
    let _cs43l22 = CS43L22::new(i2c, 0x94, Config::new()).unwrap();

    exit()
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
