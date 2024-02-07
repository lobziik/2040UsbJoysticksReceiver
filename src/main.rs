//! Rainbow effect color wheel using the onboard NeoPixel on an Waveshare RP2040 Zero board
//!
//! This flows smoothly through various colors on the onboard NeoPixel.
//! Uses the `ws2812_pio` driver to control the NeoPixel, which in turns uses the
//! RP2040's PIO block.
//!
//! Copypasted from https://github.com/rp-rs/rp-hal-boards/blob/main/boards/waveshare-rp2040-zero/examples/waveshare_rp2040_zero_neopixel_rainbow.rs
//! for experimentation purposes
#![no_std]
#![no_main]

use core::iter::once;
use embedded_hal::timer::CountDown;
use fugit::ExtU32;
use panic_halt as _;

use waveshare_rp2040_zero as bsp;
use bsp::hal::pio::PIOExt;
use bsp::hal::clocks::Clock;
use bsp::entry;

// USB Device support
use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
// USB Communications Class Device support
// SerialPort over usb
use usbd_serial::SerialPort;

// Used to demonstrate writing formatted strings
use core::fmt::Write;
use heapless::String;

// Onboard rgb LED
use smart_leds::{brightness, SmartLedsWrite};
use ws2812_pio::Ws2812;

mod rgb_wheel;

#[entry]
fn main() -> ! {
    let mut pac = bsp::pac::Peripherals::take().unwrap();

    let mut watchdog = bsp::hal::watchdog::Watchdog::new(pac.WATCHDOG);

    let clocks = bsp::hal::clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = bsp::hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut delay = timer.count_down();

    let sio = bsp::hal::Sio::new(pac.SIO);
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(bsp::hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Create a USB device with a fake VID and PID
    let mut _usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    // Configure the addressable LED
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut ws = Ws2812::new(
        // The onboard NeoPixel is attached to GPIO pin #16 on the Feather RP2040.
        pins.neopixel.into_function(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    // Infinite colour wheel loop
    let mut n: u8 = 128;
    loop {
        ws.write(brightness(once(rgb_wheel::wheel(n)), 32)).unwrap();
        n = n.wrapping_add(1);

        // let mut text: String<64> = String::new();
        // writeln!(&mut text, "Current wheel value: {}", n).unwrap();
        // let _ = serial.write(text.as_bytes());

        let _ = _usb_dev.poll(&mut [&mut serial]);

        delay.start(10.millis());
        let _ = nb::block!(delay.wait());
    }
}
