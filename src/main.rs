#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use static_cell::StaticCell;

use esp_println::println;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    peripherals::{Peripherals, UART1},
    prelude::*,
    embassy,
    uart::{Uart, TxRxPins, UartRx, config::{Config, AtCmdConfig}},
    gpio::IO,
    Async,
    i2c::{I2C},
    delay::Delay,
};
use embassy_executor::Spawner;
use embassy_time::{Ticker, Duration, Timer};
use embassy_sync::pipe::{Pipe, Writer};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

const BUF_SIZE: usize = 0x7f;

#[embassy_executor::task]
async fn reader(mut rx: UartRx<'static, UART1, Async>, pipe_writer: Writer<'static, NoopRawMutex, BUF_SIZE>) {
    loop {
        let mut buf: [u8; BUF_SIZE] = [0x00; BUF_SIZE];
        let read_size = rx.read_async(&mut buf).await.unwrap();
        pipe_writer.write(&buf[..read_size]).await;
    }
}

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

    static PIPE: StaticCell<Pipe<NoopRawMutex, BUF_SIZE>> = StaticCell::new();
    let pipe = PIPE.init(Pipe::new());
    let (pipe_reader, pipe_writer) = pipe.split();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut i2c = I2C::new_async(
        peripherals.I2C0,
        io.pins.gpio12, // sda
        io.pins.gpio14, // scl
        100.kHz(),
        &clocks,
    );

    let pins = TxRxPins::new_tx_rx(io.pins.gpio18, io.pins.gpio5);
    let mut uart = Uart::new_async_with_config(
        peripherals.UART1,
        Config::default(),
        Some(pins),
        &clocks,
    );
    uart.set_at_cmd(AtCmdConfig::new(None, None, None, b'\r', None));
    uart.set_rx_fifo_full_threshold(BUF_SIZE as u16).unwrap();
    let (mut tx, rx) = uart.split();
    spawner.must_spawn(reader(rx, pipe_writer));

    Timer::after_millis(5_000).await;
    let mut delay = Delay::new(&clocks);
    let mut lcd = match Lcd::new(&mut i2c, LCD_ADDRESS, &mut delay) {
        Ok(lcd) => lcd,
        Err(e) => {
            panic!("Error LCD: {:?}", e);
        }
    };

    // let mut ticker = Ticker::every(Duration::from_millis(100));
    loop {
        let mut buf: [u8; BUF_SIZE] = [0x00; BUF_SIZE];
        let n = pipe_reader.read(&mut buf).await;
        let s = core::str::from_utf8(&buf[..n]).unwrap().trim();
        println!("Read: {}", s);
        // tx.write_async(b"> ").await.unwrap();
        // tx.write_async(&buf).await.unwrap();
        // tx.write_async(b"\r\n").await.unwrap();
        // tx.flush_async().await.unwrap();

        lcd.set_display(Display::On).unwrap();
        lcd.set_backlight(Backlight::On).unwrap();
        lcd.clear().unwrap();
        lcd.print(s).unwrap();
        // ticker.next().await;
    }
}
