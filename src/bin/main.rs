#![no_std]
#![no_main]

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    main,
    rmt::Rmt,
    time::Rate,
};
use esp_hal_smartled::{SmartLedsAdapter, smart_led_buffer};
use smart_leds::{RGB8, SmartLedsWrite, brightness};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80))
        .expect("Failed to initialize RMT");

    let mut rmt_buffer = smart_led_buffer!(1);
    let mut led = SmartLedsAdapter::new(
        rmt.channel0,
        peripherals.GPIO8,
        &mut rmt_buffer,
    );

    let delay = Delay::new();

    // Colors to blink between
    let red: RGB8 = RGB8 { r: 255, g: 0, b: 0 };
    let off: RGB8 = RGB8 { r: 0, g: 0, b: 0 };
    let level = 10; // brightness 0-255, keep it low

    loop {
        led.write(brightness([red].into_iter(), level)).unwrap();
        delay.delay_millis(500);
        led.write(brightness([off].into_iter(), level)).unwrap();
        delay.delay_millis(500);
    }
}
