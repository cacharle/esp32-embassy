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

use embassy_sync::signal::{Signal};
// use embassy_sync::blocking_mutex::raw::{NoopRawMutex};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

// embassy futures
// embassy executor
// embassy sync


#[embassy_executor::task]
async fn print_count1(signal: &'static Signal<CriticalSectionRawMutex, u32>) {
    loop {
        let n = signal.wait().await;
        println!("received: {:?}", n);
        Timer::after(Duration::from_millis(200)).await;
    }
}

#[main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    esp_println::logger::init_logger_from_env();
    let timg0 = esp_hal::timer::TimerGroup::new_async(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    static SIGNAL: StaticCell<Signal<CriticalSectionRawMutex, u32>> = StaticCell::new();
    let signal = SIGNAL.init(Signal::new());
    spawner.spawn(print_count1(signal)).unwrap();

    let mut counter = 0;
    loop {
        signal.signal(counter);
        counter += 1;
        Timer::after(Duration::from_millis(100)).await;
    }
}
