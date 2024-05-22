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
};
use embassy_executor::Spawner;
use embassy_time::{Ticker, Duration};
// use embassy_sync::channel::{Channel, Sender};
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

    // let mut ticker = Ticker::every(Duration::from_millis(1_000));

    tx.write_async(b"I repeat everything").await.unwrap();
    tx.flush_async().await.unwrap();
    loop {
        let mut buf: [u8; BUF_SIZE] = [0x00; BUF_SIZE];
        pipe_reader.read(&mut buf).await;
        tx.write_async(b"> ").await.unwrap();
        tx.write_async(&buf).await.unwrap();
        tx.write_async(b"\r\n").await.unwrap();
        tx.flush_async().await.unwrap();
        println!("sent bytes with UART: {:?}", buf);
        // ticker.next().await;
    }
}
