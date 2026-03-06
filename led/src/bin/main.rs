#![no_std]
#![no_main]
extern crate alloc;
use alloc::boxed::Box;
use alloc::vec;
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
    options::{ColorOrder, Orientation, Rotation},
    Builder,
};
use mousefood::prelude::*;
use ratatui::{
    Terminal,
    style::*,
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();
    esp_alloc::heap_allocator!(size: 64 * 1024);

    let spi_bus = Spi::new(
        peripherals.SPI2,
        SpiConfig::default().with_frequency(Rate::from_mhz(4)),
    )
    .unwrap()
    .with_sck(peripherals.GPIO6)
    .with_mosi(peripherals.GPIO7);

    let cs = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default());
    let spi_device = ExclusiveDevice::new(spi_bus, cs, delay).unwrap();

    let dc = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    let mut rst = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let d = Delay::new();
    d.delay_millis(100);
    rst.set_high();
    d.delay_millis(200);

    let buffer = Box::leak(Box::new([0u8; 512]));
    let di = SpiInterface::new(spi_device, dc, buffer);

    let mut display = Builder::new(ST7735s, di)
        .reset_pin(rst)
        .display_size(128, 128)
        .color_order(ColorOrder::Rgb)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut Delay::new())
        .unwrap();

    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig {
        color_theme: ColorTheme::tokyo_night(),

        ..Default::default()
    });
    let mut terminal = Terminal::new(backend).unwrap();

    let mut frame_count: usize = 0;

    loop {
        let count = frame_count;
        terminal.draw(|frame| {
            let line = Line::from(vec![
                Span::styled(alloc::format!("F:{count} "), Style::new().yellow()),
                Span::styled("RED ",   Style::new().fg(Color::Red)),
                Span::styled("DIM ",   Style::new().fg(Color::Red).add_modifier(Modifier::DIM)),
                Span::styled("UNDR ",  Style::new().add_modifier(Modifier::UNDERLINED)),
                Span::styled("SLOW ",  Style::new().add_modifier(Modifier::SLOW_BLINK)),
                Span::styled("FAST ",  Style::new().add_modifier(Modifier::RAPID_BLINK)),
                Span::styled("REV ",   Style::new().add_modifier(Modifier::REVERSED)),
                Span::styled("HIDE ",  Style::new().add_modifier(Modifier::HIDDEN)),
                Span::styled("XOUT ",  Style::new().add_modifier(Modifier::CROSSED_OUT)),
                Span::styled("GHOST ", Style::new().fg(Color::DarkGray).add_modifier(Modifier::DIM | Modifier::ITALIC)),
                Span::styled("ALARM ", Style::new().fg(Color::Red).add_modifier(Modifier::RAPID_BLINK | Modifier::REVERSED)),
                Span::styled("DEAD ",  Style::new().fg(Color::Gray).add_modifier(Modifier::CROSSED_OUT | Modifier::DIM)),
                Span::styled("SHOUT ", Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
                Span::styled("HAUNT ", Style::new().fg(Color::Magenta).add_modifier(Modifier::SLOW_BLINK | Modifier::DIM)),
                Span::styled("CRIT",   Style::new().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK)),
            ]);

            let paragraph = Paragraph::new(vec![line]).wrap(Wrap { trim: true });
            let block = Block::bordered()
                .border_style(Style::new().yellow())
                .title("Mods");
            frame.render_widget(paragraph.block(block), frame.area());
        }).unwrap();

        frame_count = frame_count.wrapping_add(1);
    }
}
