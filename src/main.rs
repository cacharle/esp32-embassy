#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use static_cell::StaticCell;

use esp_println::println;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    prelude::*,
    embassy,
};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};


// embassy executor

use embassy_futures::{join::join, select::{Either, select}};
use embassy_time::Ticker;

#[main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    esp_println::logger::init_logger_from_env();
    let timg0 = esp_hal::timer::TimerGroup::new_async(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    let mut ticker1 = Ticker::every(Duration::from_millis(1000));
    let mut ticker2 = Ticker::every(Duration::from_millis(1500));
    loop {
        match select(ticker1.next(), ticker2.next()).await {
            Either::First(_) => println!("first future finished"),
            Either::Second(_) => println!("second future finished"),
        }
    }
}
