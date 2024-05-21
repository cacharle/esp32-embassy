#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use esp_println::println;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    prelude::*,
    embassy,
    gpio::{IO, Output, PushPull, GpioPin},
};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

// #[embassy_executor::task]
// async fn one_second_task() {
//     // let mut count = 0;
//     loop {
//         // println!("spawn task count: {}", count);
//         println!("test!!!");
//         // count += 1;
//         Timer::after(Duration::from_millis(1_000)).await;
//     }
// }

#[embassy_executor::task]
async fn blinker(mut led: GpioPin<Output<PushPull>, 5>, interval: Duration) {
    loop {
        led.set_high();
        println!("set high");
        Timer::after(interval).await;
        led.set_low();
        println!("set low");
        Timer::after(interval).await;
    }
}

#[main]
async fn main(spawner: Spawner) -> ! {
    println!("Init!");
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    esp_println::logger::init_logger_from_env();

    let timg0 = esp_hal::timer::TimerGroup::new_async(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let led = io.pins.gpio5.into_push_pull_output(); // ::new(io.pins.gpio5);

    spawner.spawn(blinker(led, Duration::from_millis(1_000))).unwrap();

    loop {
        println!("HELLO FROM MAIN");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}
