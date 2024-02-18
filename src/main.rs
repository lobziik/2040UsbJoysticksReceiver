#![no_std]
#![no_main]

#[cfg(all(feature = "rp-pico", feature = "waveshare-rp2040-zero"))]
compile_error!(
    "board specific crate \"rp-pico\" and \"waveshare-rp2040-zero\" cannot be enabled at the same time"
);

#[cfg(feature = "rp-pico")]
use rp_pico as bsp;

#[cfg(feature = "waveshare-rp2040-zero")]
use waveshare_rp2040_zero as bsp;

use defmt_rtt as _; // logger
use panic_halt as _;

use embedded_hal::timer::CountDown;
use fugit::ExtU32;

// The macro for our start-up function
use bsp::entry;
use bsp::hal;

// The macro for marking our interrupt functions
use bsp::hal::pac::interrupt;
use embedded_hal::digital::v2::OutputPin;

// Spi setup related traits
use bsp::hal::fugit::RateExtU32;
use hal::clocks::Clock;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::hid_class::HIDClass;
use usdb_joystic_hid_descriptor::JoystickReport;

mod xn297;

/// The USB Device Driver (shared with the interrupt).
static mut USB_DEVICE: Option<UsbDevice<hal::usb::UsbBus>> = None;

/// The USB Bus Driver (shared with the interrupt).
static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;

/// The USB Human Interface Device Driver (shared with the interrupt).
static mut USB_HID_JOY_P1: Option<HIDClass<hal::usb::UsbBus>> = None;
/// The USB Human Interface Device Driver (shared with the interrupt).
static mut USB_HID_JOY_P2: Option<HIDClass<hal::usb::UsbBus>> = None;

enum Player {
    One,
    Two,
}

#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
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
    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let spi_miso = pins.gpio16.into_function::<hal::gpio::FunctionSpi>();
    // seems doesnt work in a way i need, experiment
    // https://github.com/rp-rs/rp-hal/issues/480
    // let spi_csn = pins.gpio17.into_function::<hal::gpio::FunctionSpi>();
    let spi_sclk = pins.gpio18.into_function::<hal::gpio::FunctionSpi>();
    let spi_mosi = pins.gpio19.into_function::<hal::gpio::FunctionSpi>();

    let mut spi_csn = pins.gpio17.into_push_pull_output();
    spi_csn.set_high().unwrap();

    let mut spi_ce = pins.gpio21.into_push_pull_output();
    spi_ce.set_low().unwrap();

    let spi = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        4.MHz(),
        embedded_hal::spi::MODE_0,
    );

    let mut t = xn297::Xn297L::new(spi, spi_csn, spi_ce);
    t.init().unwrap();

    let usb_bus = hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    );

    let usb_bus_allocator = UsbBusAllocator::new(usb_bus);

    unsafe {
        // Note (safety): This is safe as interrupts haven't been started yet
        USB_BUS = Some(usb_bus_allocator);
    }

    // Grab a reference to the USB Bus allocator. We are promising to the
    // compiler not to take mutable access to this global variable whilst this
    // reference exists!
    let bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

    // // Set up the USB HID Class Device driver, providing Mouse Reports
    let usb_hid_j1 = HIDClass::new(bus_ref, JoystickReport::desc(), 10);
    let usb_hid_j2 = HIDClass::new(bus_ref, JoystickReport::desc(), 10);
    unsafe {
        // Note (safety): This is safe as interrupts haven't been started yet.
        USB_HID_JOY_P1 = Some(usb_hid_j1);
        USB_HID_JOY_P2 = Some(usb_hid_j2);
    }

    // // Create a USB device with a fake VID and PID
    let usb_dev = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x16c0, 0x27da))
        .manufacturer("Kachnamalir")
        .product("Rusty radio joysticks")
        .serial_number("one-of-a-kind-pico")
        .device_class(0)
        .composite_with_iads()
        .build();

    unsafe {
        // Note (safety): This is safe as interrupts haven't been started yet
        USB_DEVICE = Some(usb_dev);
    }

    unsafe {
        // Enable the USB interrupt
        hal::pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ);
    };

    let mut input_count_down = timer.count_down();
    input_count_down.start(9_u32.millis());

    let mut check_transmission = timer.count_down();
    check_transmission.start(10_u32.millis());

    let player_one_report = JoystickReport {
        x: 0,
        y: 0,
        buttons: 0,
    };
    let player_two_report = JoystickReport {
        x: 0,
        y: 0,
        buttons: 0,
    };

    loop {
        if check_transmission.wait().is_ok() {
            // 3 because register is a part of transmission, need fix it
            let _ = t.read_rx_payload::<3>().unwrap().is_some_and(|payload| {
                let [_, first_byte, second_byte] = payload;
                defmt::println!("payload: {:#010b} {:#010b}", first_byte, second_byte);
                true
            });
        }

        if input_count_down.wait().is_ok() {
            match push_joystick_report(player_one_report, Player::One) {
                Err(UsbError::WouldBlock) => {}
                Ok(_) => {}
                Err(e) => {
                    defmt::println!("err: {:?}", e)
                }
            }

            match push_joystick_report(player_two_report, Player::Two) {
                Err(UsbError::WouldBlock) => {}
                Ok(_) => {}
                Err(e) => {
                    defmt::println!("err: {:?}", e)
                }
            }
        }
    }
}

fn push_joystick_report(report: JoystickReport, player: Player) -> Result<usize, UsbError> {
    critical_section::with(|_| unsafe {
        match player {
            Player::One => USB_HID_JOY_P1.as_mut().map(|hid| hid.push_input(&report)),
            Player::Two => USB_HID_JOY_P2.as_mut().map(|hid| hid.push_input(&report)),
        }
    })
    .unwrap()
}

/// This function is called whenever the USB Hardware generates an Interrupt
/// Request.
#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    // Handle USB request
    let usb_dev = USB_DEVICE.as_mut().unwrap();
    let usb_hid_j1 = USB_HID_JOY_P1.as_mut().unwrap();
    let usb_hid_j2 = USB_HID_JOY_P2.as_mut().unwrap();
    usb_dev.poll(&mut [usb_hid_j1, usb_hid_j2]);
}
