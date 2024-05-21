#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::borrow::BorrowMut;
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

use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::blocking_mutex::raw::{NoopRawMutex};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

// embassy futures
// embassy executor
// embassy sync

// Channel<NoopRawMutex, u32, 3>

// static CHANNEL: StaticCell<Channel::<NoopRawMutex, u32, 3>> = StaticCell::new();

#[embassy_executor::task]
async fn print_count1(id: i32, channel: Receiver<'static, NoopRawMutex, u32, 3>) {
    loop {
        let n = channel.receive().await;
        println!("received in {}: {}", id, n);
    }
}

#[embassy_executor::task]
async fn print_count2(id: i32, channel: Receiver<'static, NoopRawMutex, u32, 3>) {
    loop {
        let n = channel.receive().await;
        println!("received in {}: {}", id, n);
    }
}

#[embassy_executor::task]
async fn produce_good_numbers(channel: Sender<'static, NoopRawMutex, u32, 3>) {
    let mut counter = 1000;
    loop {
        channel.send(counter).await;
        counter += 1;
        Timer::after(Duration::from_millis(300)).await;
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

    static CHANNEL: StaticCell<Channel::<NoopRawMutex, u32, 3>> = StaticCell::new();
    let channel = CHANNEL.init(Channel::<NoopRawMutex, u32, 3>::new());

    let sender = channel.sender();
    let receiver = channel.receiver();

    spawner.spawn(print_count1(1, receiver.clone())).unwrap();
    spawner.spawn(print_count2(2, receiver)).unwrap();
    spawner.spawn(produce_good_numbers(sender)).unwrap();

    let mut counter = 0;
    loop {
        sender.send(counter).await;
        counter += 1;
        Timer::after(Duration::from_millis(500)).await;
    }
}
