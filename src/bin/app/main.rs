#![no_std]
#![no_main]

use esp32_hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Rtc,
    Delay,
    spi::{Spi, SpiMode},
    gpio::IO,
};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};

use esp_backtrace as _;
use esp_println::println;
use nb::block;
use st7735_lcd;
use st7735_lcd::Orientation;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks);
    let mut timer0 = timer_group0.timer0;
    let mut wdt = timer_group0.wdt;
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    wdt.disable();
    rtc.rwdt.disable();

    timer0.start(1u64.secs());

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let sck = io.pins.gpio18;
    let sda = io.pins.gpio23;
    let miso = io.pins.gpio19.into_push_pull_output();
    let cs = io.pins.gpio5;

    let dc = io.pins.gpio13.into_push_pull_output();
    let rst = io.pins.gpio14.into_push_pull_output();

    let spi = Spi::new(
        peripherals.SPI2,
        sck,
        sda,
        dc,
        cs,
        12u32.MHz(),
        SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let mut disp = st7735_lcd::ST7735::new(spi, miso, rst, true, false, 160, 128);
    let mut delay = Delay::new(&clocks);

    disp.init(&mut delay).unwrap();
    disp.set_orientation(&Orientation::Landscape).unwrap();
    disp.clear(Rgb565::BLACK).unwrap();
    disp.set_offset(0, 0);


    loop {
        println!("Hello world!");
        block!(timer0.wait()).unwrap();
        disp.clear(Rgb565::RED).unwrap();
        block!(timer0.wait()).unwrap();
        disp.clear(Rgb565::BLUE).unwrap();
        // Draw centered text.
        Text::new("Hello Rust!", Point::new(20, 30), style).draw(&mut disp).unwrap();
    }
}