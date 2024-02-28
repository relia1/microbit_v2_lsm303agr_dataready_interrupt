#![no_std]

use panic_rtt_target as _;

use crate::twim::Twim;
use core::convert::Into;
use lsm303agr::{interface::I2cInterface, mode::MagOneShot, Interrupt, Lsm303agr};

use microbit::{
    board::Board,
    hal::{delay::Delay, gpiote::Gpiote, prelude::*, twim, Timer},
    pac::{self as pac, twim0::frequency::FREQUENCY_A, TIMER0},
};

pub enum BoardState {
    Falling,
    NotFalling,
}

pub struct MB2 {
    pub sensor: Lsm303agr<I2cInterface<Twim<pac::TWIM0>>, MagOneShot>,
    pub timer: Timer<TIMER0>,
    pub state: BoardState,
    pub gpiote: Gpiote,
}

impl MB2 {
    pub fn new() -> Result<Self, &'static str> {
        if let Some(mut board) = Board::take() {
            let gpiote = Gpiote::new(board.GPIOTE);
            let mut timer = Timer::new(board.TIMER0);
            let state = BoardState::NotFalling;
            let mut i2c =
                twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);
            let mut delay = Delay::new(board.SYST);
            let mut buf: [u8; 5] = [0, 0, 0, 0, 0];
            let _ = i2c.read(0x70, &mut buf);
            delay.delay_ms(1000u16);
            let mut sensor = Lsm303agr::new_with_i2c(i2c);

            sensor.init().unwrap();
            sensor
                .set_accel_mode_and_odr(
                    &mut timer,
                    lsm303agr::AccelMode::LowPower,
                    lsm303agr::AccelOutputDataRate::Hz100,
                )
                .unwrap();

            sensor.acc_enable_interrupt(Interrupt::DataReady1).unwrap();

            gpiote
                .channel0()
                .input_pin(&board.pins.p0_25.degrade().into_pullup_input())
                .hi_to_lo()
                .enable_interrupt();
            gpiote.channel0().reset_events();

            unsafe {
                board.NVIC.set_priority(pac::Interrupt::GPIOTE, 128);
                pac::NVIC::unmask(pac::Interrupt::GPIOTE);
            }

            pac::NVIC::unpend(pac::Interrupt::GPIOTE);

            Ok(MB2 {
                sensor,
                timer,
                state,
                gpiote,
            })
        } else {
            Err("Board not available")
        }
    }

    pub fn get_accel_data(&mut self) -> (f32, f32, f32) {
        let accel_reading = self.sensor.acceleration().unwrap();
        let (x, y, z) = accel_reading.xyz_mg();
        (
            (x as f32) / 1000.0,
            (y as f32) / 1000.0,
            (z as f32) / 1000.0,
        )
    }
}
