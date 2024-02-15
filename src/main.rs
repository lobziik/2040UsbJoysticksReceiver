#![no_std]
#![no_main]

use rp_pico as bsp;

use defmt_rtt as _; // logger
use panic_halt as _;

// The macro for our start-up function
use bsp::entry;
use bsp::hal;
use bsp::hal::prelude::*;
// The macro for marking our interrupt functions
use bsp::hal::pac::interrupt;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::hid_class::HIDClass;
use usdb_joystic_hid_descriptor::JoystickReport;


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

/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialised.
///
/// The function configures the RP2040 peripherals, then submits cursor movement
/// updates periodically.
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

    {
        let sio = hal::Sio::new(pac.SIO);
        let _pins = rp_pico::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );
    }

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
    let usb_hid_j1 = HIDClass::new(bus_ref, JoystickReport::desc(), 20);
    let usb_hid_j2 = HIDClass::new(bus_ref, JoystickReport::desc(), 20);
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
    let core = hal::pac::CorePeripherals::take().unwrap();
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    loop {
        delay.delay_ms(1000);
        let report = JoystickReport {
            x: 0,
            y: 0,
            buttons: 1,
        };
        let _ = match push_joystick_report(report, Player::One) {
            Ok(size) => {
                defmt::println!("{} sent!", size)
            }
            Err(err) => {
                defmt::println!("err {}", err);
            }
        };
        defmt::println!("tick")
    }
}

fn push_joystick_report(
    report: JoystickReport,
    player: Player,
) -> Result<usize, usb_device::UsbError> {
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
