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
use embassy_sync::pipe::{Pipe, Reader, Writer};

// embassy futures
// embassy executor
// embassy sync

use embedded_io_async::{Read, Write};


#[embassy_executor::task]
async fn print_count1(mut reader: Reader<'static, NoopRawMutex, 10>) {
    loop {
        let mut buf: [u8; 20] = [0x00; 20];
        reader.read_exact(&mut buf).await.unwrap();
        println!("read {:?}", buf);
        Timer::after(Duration::from_millis(500)).await;
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

    static PIPE: StaticCell<Pipe<NoopRawMutex, 10>> = StaticCell::new();
    let pipe = PIPE.init(Pipe::new());

    let (reader, mut writer) = pipe.split();


    spawner.spawn(print_count1(reader)).unwrap();

    loop {
        let buf: [u8; 7] = [1, 2, 3, 4, 5, 6, 7];
        writer.write_all(&buf[..]).await.unwrap();
        Timer::after(Duration::from_millis(200)).await;
    }
}
