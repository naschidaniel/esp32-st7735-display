#![no_std]
#![no_main]

use core::fmt::Write;
use ee895::EE895;
use embedded_graphics::image::{Image, ImageRaw, ImageRawLE};
use embedded_graphics::{
    mono_font::{ascii::FONT_9X15, ascii::FONT_9X15_BOLD, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use esp32_hal::{
    clock::ClockControl,
    gpio::IO,
    i2c::I2C,
    peripherals::Peripherals,
    prelude::*,
    spi::{Spi, SpiMode},
    timer::TimerGroup,
    Delay, Rtc,
};
use esp_backtrace as _;
use esp_println::println;
use heapless::String;
use st7735_lcd;
use st7735_lcd::Orientation;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);

    // init Watchdog and RTC
    let mut wdt = timer_group0.wdt;
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    rtc.rwdt.disable();
    wdt.start(10u64.secs());

    // delay
    let mut delay = Delay::new(&clocks);

    // Embedded Graphics
    let style = MonoTextStyle::new(&FONT_9X15, Rgb565::WHITE);
    let text_style_big = MonoTextStyle::new(&FONT_9X15_BOLD, Rgb565::WHITE);

    //
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // I2C Sensor Settings
    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21,
        io.pins.gpio22,
        10u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let mut sensor = EE895::new(i2c).unwrap();
    let mut warning: &str;
    let mut color: Rgb565;
    let mut co2: f32;
    let mut temperature: f32;
    let mut pressure: f32;

    println!("Hello World!");

    // onboard LED
    let mut led = io.pins.gpio2.into_push_pull_output();
    // SPI Display Settings
    let sck = io.pins.gpio18; // sck
    let sda = io.pins.gpio23; // sda
    let miso = io.pins.gpio19.into_push_pull_output(); // A0
    let cs = io.pins.gpio5; // CS
    let dc = io.pins.gpio13.into_push_pull_output(); // dc
    let rst = io.pins.gpio14.into_push_pull_output();

    let spi = Spi::new(
        peripherals.SPI2,
        sck,
        sda,
        dc,
        cs,
        60u32.MHz(),
        SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let mut display = st7735_lcd::ST7735::new(spi, miso, rst, true, false, 160, 128);

    display.init(&mut delay).unwrap();
    display.set_orientation(&Orientation::Landscape).unwrap();
    display.clear(Rgb565::BLACK).unwrap();
    display.set_offset(0, 0);

    let image_raw: ImageRawLE<Rgb565> =
        ImageRaw::new(include_bytes!("../../assets/ferris.raw"), 86);
    display.clear(Rgb565::BLACK).unwrap();
    let image: Image<_> = Image::new(&image_raw, Point::new(34, 30));
    image.draw(&mut display).unwrap();

    delay.delay_ms(4000u32);

    let mut co2_msg: String<20> = String::new();
    let mut pressure_msg: String<20> = String::new();
    let mut temperature_msg: String<20> = String::new();

    loop {
        wdt.feed();
        led.set_high().unwrap();

        co2 = sensor.read_co2().unwrap();
        temperature = sensor.read_temperature().unwrap();
        pressure = sensor.read_pressure().unwrap();

        (warning, color) = match co2 {
            v if v <= 450.0 => ("Fresh", Rgb565::BLUE),
            v if v <= 700.0 => ("Good", Rgb565::GREEN),
            v if v <= 1000.0 => ("Moderate", Rgb565::CSS_ORANGE),
            v if v <= 1500.0 => ("Unhealthy", Rgb565::CSS_INDIAN_RED),
            v if v <= 2500.0 => ("Dangerous", Rgb565::CSS_VIOLET),
            _ => ("Hazardous", Rgb565::CSS_DARK_VIOLET),
        };

        display.clear(color).unwrap();

        co2_msg.clear();
        temperature_msg.clear();
        pressure_msg.clear();
        write!(co2_msg, "CO2: {co2} ppm").unwrap();
        write!(temperature_msg, "Temperature: {temperature} C").unwrap();
        write!(pressure_msg, "Pressure: {pressure} hPa").unwrap();

        println!("{}", co2_msg);
        println!("Warning: {}", warning);
        println!("{}", temperature_msg);
        println!("{}", pressure_msg);

        Text::new(co2_msg.as_str(), Point::new(20, 30), text_style_big)
            .draw(&mut display)
            .unwrap();

        Text::new(warning, Point::new(20, 55), text_style_big)
            .draw(&mut display)
            .unwrap();

        Text::new(temperature_msg.as_str(), Point::new(20, 80), style)
            .draw(&mut display)
            .unwrap();

        Text::new(pressure_msg.as_str(), Point::new(20, 105), style)
            .draw(&mut display)
            .unwrap();

        led.set_low().unwrap();
        // Wait 5 seconds
        delay.delay_ms(5000u32);
    }
}
