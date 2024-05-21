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

use embassy_sync::blocking_mutex::raw::{NoopRawMutex};
use embassy_sync::mutex::Mutex;

// embassy futures
// embassy executor
// embassy sync


#[embassy_executor::task]
async fn print_count1(mutex: &'static Mutex<NoopRawMutex, u32>) {
    loop {
        {
            let mut counter = mutex.lock().await;
            *counter = counter.wrapping_add(100);
            // *counter += 100;
            println!("counter: {:?}", *counter);
            Timer::after(Duration::from_millis(200)).await;
        }
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

    static MUTEX: StaticCell<Mutex<NoopRawMutex, u32>> = StaticCell::new();
    let mutex = MUTEX.init(Mutex::new(0));

    spawner.spawn(print_count1(mutex)).unwrap();

    loop {
        {
            let mut counter = mutex.lock().await;
            *counter += 1;
        }
        // signal.signal(counter);
        // counter += 1;
        Timer::after(Duration::from_millis(100)).await;
    }
}
