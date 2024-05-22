#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use static_cell::StaticCell;

use esp_println::println;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    peripherals::{Peripherals},
    prelude::*,
    embassy,
    // uart::{Uart, TxRxPins, UartRx, config::{Config, AtCmdConfig}},
    gpio::IO,
    // Async,
    i2c::{I2C},
    delay::Delay,
};
use embassy_executor::Spawner;
use embassy_time::{Ticker, Duration, Timer};
// // use embassy_sync::channel::{Channel, Sender};
// use embassy_sync::pipe::{Pipe, Writer};
// use embassy_sync::blocking_mutex::raw::NoopRawMutex;

// const BUF_SIZE: usize = 0x7f;

// #[embassy_executor::task]
// async fn reader(mut rx: UartRx<'static, UART1, Async>, pipe_writer: Writer<'static, NoopRawMutex, BUF_SIZE>) {
//     loop {
//         let mut buf: [u8; BUF_SIZE] = [0x00; BUF_SIZE];
//         let read_size = rx.read_async(&mut buf).await.unwrap();
//         pipe_writer.write(&buf[..read_size]).await;
//     }
// }

use liquidcrystal_i2c_rs::{Backlight, Display, Lcd};

const LCD_ADDRESS: u8 = 39; // Had to do a for i in 0..127 {} to find the correct address

#[main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    esp_println::logger::init_logger_from_env();
    let timg0 = esp_hal::timer::TimerGroup::new_async(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut i2c = I2C::new_async(
        peripherals.I2C0,
        io.pins.gpio12, // sda
        io.pins.gpio14, // scl
        100.kHz(),
        &clocks,
    );

    let mut ticker = Ticker::every(Duration::from_millis(100));
    loop {
        let mut delay = Delay::new(&clocks);
        let mut lcd = match Lcd::new(&mut i2c, LCD_ADDRESS, &mut delay) {
            Ok(lcd) => lcd,
            Err(e) => {
                println!("Error LCD: {:?}", e);
                continue;
            }
        };
        lcd.set_display(Display::On).unwrap();
        lcd.set_backlight(Backlight::On).unwrap();
        lcd.clear().unwrap();
        lcd.print("Hello World!").unwrap();
        ticker.next().await;
    }
}
