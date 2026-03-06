#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
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
    options::ColorOrder,
    Builder,
};
use mousefood::prelude::*;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    esp_alloc::heap_allocator!(size: 64 * 1024);

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
    let d = Delay::new();
    d.delay_millis(100);   // hold reset low
    rst.set_high();
    d.delay_millis(200);   // wait for display to fully come up

    // Display interface
    let buffer = Box::leak(Box::new([0u8; 512]));
    let di = SpiInterface::new(spi_device, dc, buffer);

    let mut display = Builder::new(ST7735s, di)
        .reset_pin(rst)
        .display_size(128, 128)
        .color_order(ColorOrder::Rgb)
        .init(&mut Delay::new())
        .unwrap();

    // Setup Mousefood and Ratatui
    let backend = EmbeddedBackend::new(&mut display, Default::default());
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            let block = Block::bordered().title("Mousefood");
            let paragraph = Paragraph::new("Hello from Mousefood!").block(block);
            frame.render_widget(paragraph, frame.area());
        })
        .unwrap();

    loop {}
}
