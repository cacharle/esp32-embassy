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
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

// embassy executor

struct Message {
    count: u32,
    id: u32,
}

#[embassy_executor::task(pool_size = 10)]
async fn sending_numbers(id: u32, sender: Sender<'static, NoopRawMutex, Message, 3>) {
    let mut counter = 0;
    loop {
        let message = Message { count: counter, id };
        sender.send(message).await;
        counter += 1;
        Timer::after_millis(500).await;
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

    static CHANNEL: StaticCell<Channel::<NoopRawMutex, Message, 3>> = StaticCell::new();
    let channel = CHANNEL.init(Channel::new());
    let sender = channel.sender();
    let receiver = channel.receiver();

    spawner.spawn(sending_numbers(1, sender)).unwrap();
    spawner.spawn(sending_numbers(2, sender)).unwrap();
    spawner.spawn(sending_numbers(3, sender)).unwrap();

    loop {
        match receiver.receive().await {
            Message { count: counter, id: 1 } => println!("first task: {}", counter),
            Message { count: counter, id: 2 } => println!("second task: {}", counter),
            Message { count: counter, id: 3 } => println!("third task: {}", counter),
            _ => panic!("no id"),
        }
    }
}
