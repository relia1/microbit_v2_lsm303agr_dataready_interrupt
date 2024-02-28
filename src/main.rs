#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m::asm;
use cortex_m_rt::entry;

use drop::MB2;
use microbit::pac::interrupt;

use critical_section_lock_mut::LockMut;

static MB2_ACCEL: LockMut<MB2> = LockMut::new();

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mb2_board = MB2::new().unwrap();
    MB2_ACCEL.init(mb2_board);

    // removing this acceleration read makes this stop working for reasons
    // I have not yet looked into
    MB2_ACCEL.with_lock(|cs| {
        let (a, b, c) = cs.get_accel_data();
        rprintln!("{} {} {} priming the pump?\n", a, b, c);
    });

    loop {
        asm::wfi();
    }
}

#[interrupt]
fn GPIOTE() {
    MB2_ACCEL.with_lock(|mb2| {
        let new_data = mb2.sensor.accel_status().unwrap().xyz_new_data();
        match new_data {
            false => {
                panic!(
                    "No new data, was event triggered: {}\n",
                    mb2.gpiote.channel0().is_event_triggered()
                );
            }
            true => {
                let (a, b, c) = mb2.get_accel_data();
                rprintln!("x,y,z: {} {} {}\n", a, b, c);
            }
        };
        mb2.gpiote.channel0().reset_events();
    });
}
