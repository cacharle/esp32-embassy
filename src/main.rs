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
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};
use embassy_sync::blocking_mutex::raw::{NoopRawMutex};

// embassy futures
// embassy executor
// embassy sync

// Channel<NoopRawMutex, u32, 3>

// static CHANNEL: StaticCell<Channel::<NoopRawMutex, u32, 3>> = StaticCell::new();

const SUBS: usize = 100;
const PUBS: usize = 100;
type U32PubSub = PubSubChannel<NoopRawMutex, u32, 1, SUBS, PUBS>;

#[embassy_executor::task]
async fn print_count1(id: i32, pubsub: &'static U32PubSub) {
    let mut sub = pubsub.subscriber().unwrap();
    loop {
        Timer::after(Duration::from_millis(3000)).await;
        let n = sub.next_message().await;
        println!("received in {}: {:?}", id, n);
    }
}

#[embassy_executor::task]
async fn print_count2(id: i32, pubsub: &'static U32PubSub) {
    let mut sub = pubsub.subscriber().unwrap();
    loop {
        Timer::after(Duration::from_millis(2000)).await;
        let n = sub.next_message().await;
        println!("received in {}: {:?}", id, n);
    }
}


#[embassy_executor::task]
async fn produce_good_numbers(pubsub: &'static U32PubSub) {
    let pub1 = pubsub.publisher().unwrap();
    let mut counter = 1000;
    loop {
        pub1.publish(counter).await;
        counter += 1;
        Timer::after(Duration::from_millis(10)).await;
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

    static PUBSUB: StaticCell<U32PubSub> = StaticCell::new();
    let pubsub = PUBSUB.init(PubSubChannel::new());


    spawner.spawn(print_count1(1, pubsub)).unwrap();
    spawner.spawn(print_count2(2, pubsub)).unwrap();
    spawner.spawn(produce_good_numbers(pubsub)).unwrap();

    let pub0 = pubsub.publisher().unwrap();

    let mut counter = 0;
    loop {
        pub0.publish(counter).await;
        counter += 1;
        Timer::after(Duration::from_millis(10)).await;
    }
}
