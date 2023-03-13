#![no_main]
#![no_std]

use cs43l22::{Config, CS43L22};
use defmt_rtt as _;
use panic_probe as _;
use stm32f4xx_hal::{
    block,
    i2s::{self, stm32_i2s_v12x::transfer::*},
    pac,
    prelude::*,
};

const SAMPLE_RATE: u32 = 48_000;

/// A sine wave spanning 64 samples
///
/// With a sample rate of 48 kHz, this produces a 750 Hz tone.
const SINE_750: [i16; 64] = [
    0, 3211, 6392, 9511, 12539, 15446, 18204, 20787, 23169, 25329, 27244, 28897, 30272, 31356,
    32137, 32609, 32767, 32609, 32137, 31356, 30272, 28897, 27244, 25329, 23169, 20787, 18204,
    15446, 12539, 9511, 6392, 3211, 0, -3211, -6392, -9511, -12539, -15446, -18204, -20787, -23169,
    -25329, -27244, -28897, -30272, -31356, -32137, -32609, -32767, -32609, -32137, -31356, -30272,
    -28897, -27244, -25329, -23169, -20787, -18204, -15446, -12539, -9511, -6392, -3211,
];

/// A sine wave spanning 128 samples
///
/// With a sample rate of 48 kHz, this produces a 375 Hz tone.
const SINE_375: [i16; 128] = [
    0, 1607, 3211, 4807, 6392, 7961, 9511, 11038, 12539, 14009, 15446, 16845, 18204, 19519, 20787,
    22004, 23169, 24278, 25329, 26318, 27244, 28105, 28897, 29621, 30272, 30851, 31356, 31785,
    32137, 32412, 32609, 32727, 32767, 32727, 32609, 32412, 32137, 31785, 31356, 30851, 30272,
    29621, 28897, 28105, 27244, 26318, 25329, 24278, 23169, 22004, 20787, 19519, 18204, 16845,
    15446, 14009, 12539, 11038, 9511, 7961, 6392, 4807, 3211, 1607, 0, -1607, -3211, -4807, -6392,
    -7961, -9511, -11038, -12539, -14009, -15446, -16845, -18204, -19519, -20787, -22004, -23169,
    -24278, -25329, -26318, -27244, -28105, -28897, -29621, -30272, -30851, -31356, -31785, -32137,
    -32412, -32609, -32727, -32767, -32727, -32609, -32412, -32137, -31785, -31356, -30851, -30272,
    -29621, -28897, -28105, -27244, -26318, -25329, -24278, -23169, -22004, -20787, -19519, -18204,
    -16845, -15446, -14009, -12539, -11038, -9511, -7961, -6392, -4807, -3211, -1607,
];

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(96.MHz())
        .i2s_clk(61_440.kHz())
        .freeze();
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let gpiod = dp.GPIOD.split();

    let mut audio_reset = gpiod.pd4.into_push_pull_output();

    audio_reset.set_high();

    let pins = (gpiob.pb6, gpiob.pb9);
    let i2c = dp.I2C1.i2c(pins, 100.kHz(), &clocks);
    let mut codec = CS43L22::new(i2c, 0x4A, Config::new().volume(100).verify_write(true)).unwrap();

    codec.play().unwrap();

    let pins = (gpioa.pa4, gpioc.pc10, gpioc.pc7, gpioc.pc12);
    let i2s = i2s::I2s::new(dp.SPI3, pins, &clocks);
    let i2s_config = I2sTransferConfig::new_master()
        .transmit()
        .standard(Philips)
        .data_format(Data32Channel32)
        .request_frequency(SAMPLE_RATE);
    let mut i2s_transfer = I2sTransfer::new(i2s, i2s_config);
    defmt::println!("real sample rate: {}", i2s_transfer.sample_rate());

    let sine_375_1sec = SINE_375
        .iter()
        .map(|&x| {
            let x = (x as i32) << 16;
            (x, x)
        })
        .cycle()
        .take(SAMPLE_RATE as usize);

    let sine_750_1sec = SINE_750
        .iter()
        .map(|&x| {
            let x = (x as i32) << 16;
            (x, x)
        })
        .cycle()
        .take(SAMPLE_RATE as usize);

    loop {
        for sample in sine_375_1sec.clone() {
            block!(i2s_transfer.write(sample)).ok();
        }

        i2s_transfer.write_iter(sine_750_1sec.clone());
    }
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
