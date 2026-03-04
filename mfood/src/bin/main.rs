#![no_std]
#![no_main]

use core::fmt::Write;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
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
use grift::{ Lisp};
use mipidsi::{
    Builder,
    interface::SpiInterface,
    models::ST7735s,
    options::{ColorOrder,Orientation,Rotation},
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

    let cs = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default());
    let spi_device = ExclusiveDevice::new(spi_bus, cs, delay).unwrap();

    let dc = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    let mut rst = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let mut d = Delay::new();
    d.delay_millis(100);
    rst.set_high();
    d.delay_millis(200);

    let mut buffer = [0u8; 512];
    let di = SpiInterface::new(spi_device, dc, &mut buffer);

    let mut display = Builder::new(ST7735s, di)
        .reset_pin(rst)
        .display_size(128, 128)
        .color_order(ColorOrder::Rgb)
         .orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut Delay::new())
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();


    // smaller font fits more on 128x128
    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let yellow = MonoTextStyle::new(&FONT_6X10, Rgb565::YELLOW);
    let green  = MonoTextStyle::new(&FONT_6X10, Rgb565::GREEN);

    let lisp: Lisp<5000> = Lisp::new();
    let result = lisp.eval(r#"
        (define! make-adder
            (lambda (n)
                (vau (x) e
                    (+ n (eval x e)))))
        (define! add5 (make-adder 5))
        (add5 (+ 1 2))
    "#).unwrap();

    // format result into stack buffer
    let mut buf = [0u8; 16];
    let mut pos = 0usize;
    heapless_write(&mut buf, &mut pos, result);
    let result_str = core::str::from_utf8(&buf[..pos]).unwrap_or("?");

    display.clear(Rgb565::BLACK).unwrap();

    // Draw the raw code line by line
    Text::new("(define! make-adder", Point::new(2, 10),  style).draw(&mut display).unwrap();
    Text::new("  (lambda (n)",       Point::new(2, 22),  style).draw(&mut display).unwrap();
    Text::new("    (vau (x) e",      Point::new(2, 34),  style).draw(&mut display).unwrap();
    Text::new("      (+ n",          Point::new(2, 46),  style).draw(&mut display).unwrap();
    Text::new("       (eval x e)))))", Point::new(2, 58), style).draw(&mut display).unwrap();
    Text::new("(define! add5",       Point::new(2, 70),  style).draw(&mut display).unwrap();
    Text::new("  (make-adder 5))",   Point::new(2, 82),  style).draw(&mut display).unwrap();
    Text::new("(add5 (+ 1 2))",      Point::new(2, 94),  style).draw(&mut display).unwrap();

    // Divider
    Rectangle::new(Point::new(0, 102), Size::new(128, 1))
        .into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::CSS_GRAY).build())
        .draw(&mut display).unwrap();

    // Result
    Text::new("=>", Point::new(2, 116), yellow).draw(&mut display).unwrap();
    Text::new(result_str, Point::new(18, 116), green).draw(&mut display).unwrap();

    loop {}
}

// Write a grift Value as decimal into a byte buffer without alloc
fn heapless_write(buf: &mut [u8], cursor: &mut usize, value: impl core::fmt::Display) {
    struct BufWriter<'a> {
        buf: &'a mut [u8],
        pos: &'a mut usize,
    }
    impl core::fmt::Write for BufWriter<'_> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let space = self.buf.len() - *self.pos;
            let n = bytes.len().min(space);
            self.buf[*self.pos..*self.pos + n].copy_from_slice(&bytes[..n]);
            *self.pos += n;
            Ok(())
        }
    }
    let _ = write!(BufWriter { buf, pos: cursor }, "{}", value);
}
