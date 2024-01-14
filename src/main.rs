#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::{Executor, Spawner};
use embassy_time::{Duration, Instant, Ticker, Timer};
use esp32c6_hal::{
    clock::ClockControl,
    embassy,
    gpio::{GpioPin, InvertedOutput, OpenDrain, Output, PullUp, PushPull, IO},
    peripherals::Peripherals,
    prelude::*,
    systimer::SystemTimer,
    timer::TimerGroup,
};
use esp_backtrace as _;
use esp_println::println;
use lazy_static::lazy_static;
use smart_leds::{brightness, Brightness, RGB8};
use static_cell::StaticCell;

mod ws2812_driver;
use ws2812_driver::Ws2812;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    // let timer = TimerGroup::new(peripherals.TIMG0, &clocks).timer0;
    let timer = SystemTimer::new(peripherals.SYSTIMER);
    embassy::init(&clocks, timer);

    // setup logger
    // To change the log_level change the env section in .cargo/config.toml
    // or remove it and set ESP_LOGLEVEL manually before running cargo run
    // this requires a clean rebuild because of https://github.com/rust-lang/cargo/issues/10358
    esp_println::logger::init_logger_from_env();
    log::info!("Logger is setup");
    println!("Hello world!");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    // GPIO 4 as output
    let led = io.pins.gpio4.into_push_pull_output();

    let rgb = io.pins.gpio8.into_open_drain_output();
    let ws_driver = ws2812_driver::Ws2812::new(rgb);

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(blink(led)).ok();
        spawner.spawn(onboard_rgb_led(ws_driver)).ok();
    })
}

#[embassy_executor::task]
async fn blink(mut led: GpioPin<Output<PushPull>, 4>) {
    let mut delay = Ticker::every(Duration::from_secs(1));
    loop {
        led.toggle().unwrap();
        delay.next().await;
    }
}

#[embassy_executor::task]
async fn onboard_rgb_led(mut rgb_driver: ws2812_driver::Ws2812<GpioPin<Output<OpenDrain>, 8>>) {
    rgb_driver.reset();

    Timer::after_secs(2).await;
    loop {
        log::info!("Off");
        rgb_driver
            .write([RGB8::from((0, 0, 0)); 1].iter().cloned())
            .ok();
        Timer::after_secs(2).await;

        log::info!("Red");
        rgb_driver
            .write([RGB8::from((32, 0, 0)); 1].iter().cloned())
            .ok();
        Timer::after_secs(2).await;

        log::info!("Red/Green");
        rgb_driver
            .write([RGB8::from((32, 32, 0)); 1].iter().cloned())
            .ok();
        Timer::after_secs(2).await;

        log::info!("Green");
        rgb_driver
            .write([RGB8::from((0, 32, 0)); 1].iter().cloned())
            .ok();
        Timer::after_secs(2).await;

        log::info!("Green/Blue");
        rgb_driver
            .write([RGB8::from((0, 32, 32)); 1].iter().cloned())
            .ok();
        Timer::after_secs(2).await;

        log::info!("Blue");
        rgb_driver
            .write([RGB8::from((0, 0, 32)); 1].iter().cloned())
            .ok();
        Timer::after_secs(2).await;

        log::info!("White");
        rgb_driver
            .write([RGB8::from((32, 32, 32)); 1].iter().cloned())
            .ok();
        Timer::after_secs(2).await;
    }
}
