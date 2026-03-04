#![no_std]
#![no_main]

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main,
    spi::master::{Config as SpiConfig, Spi},
    time::Rate,
};
use mipidsi::{
    interface::SpiInterface,
    models::ST7735s,
    options::{ColorInversion, ColorOrder},
    Builder,
};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    // SPI bus
    let spi_bus = Spi::new(
        peripherals.SPI2,
        SpiConfig::default().with_frequency(Rate::from_mhz(4)),
    )
    .unwrap()
    .with_sck(peripherals.GPIO6)
    .with_mosi(peripherals.GPIO7);

    // CS pin — GPIO10, wraps SpiBus into SpiDevice
    let cs = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default());
    let spi_device = ExclusiveDevice::new(spi_bus, cs, delay).unwrap();

    // DC (RS) and RST pins
    let dc = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    let mut rst = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());  // start LOW
    let mut d = Delay::new();
    d.delay_millis(100);   // hold reset low
    rst.set_high();
    d.delay_millis(200);   // wait for display to fully come up

    // Display interface
    let mut buffer = [0u8; 512];
    let di = SpiInterface::new(spi_device, dc, &mut buffer);

    let mut display = Builder::new(ST7735s, di)
        .reset_pin(rst)
        .display_size(128, 128)
        .color_order(ColorOrder::Rgb)          // swap Bgr → Rgb
        // remove invert_colors entirely
        .init(&mut Delay::new())
        .unwrap();
    // Black background
    display.clear(Rgb565::RED).unwrap();

    // Red border
    Rectangle::new(Point::new(5, 5), Size::new(118, 118))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_color(Rgb565::RED)
                .stroke_width(3)
                .build(),
        )
        .draw(&mut display)
        .unwrap();

    // Hello World text
    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    Text::new("Hello,", Point::new(14, 50), text_style)
        .draw(&mut display)
        .unwrap();
    Text::new("World!", Point::new(14, 75), text_style)
        .draw(&mut display)
        .unwrap();

    loop {}
}
