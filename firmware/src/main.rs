// Simple keyboard firmware. Inspired by the RustyKeys project:
// https://github.com/KOBA789/rusty-keys/blob/main/firmware/keyboard/src/bin/simple.rs

#![no_main]
#![no_std]

use core::convert::Infallible;
use cortex_m::delay::Delay;
use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    timer::CountDown,
};
use embedded_time::duration::Extensions;
use panic_reset as _;
use rp2040_hal::{usb::UsbBus, Clock, Watchdog};
use usb_device::{bus::UsbBusAllocator, device::UsbDeviceBuilder, prelude::UsbVidPid};
use usbd_hid::{
    descriptor::{KeyboardReport, SerializedDescriptor},
    hid_class::{
        HIDClass, HidClassSettings, HidCountryCode, HidProtocol, HidSubClass, ProtocolModeConfig,
    },
};

use rp2040_hal::pac;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const EXTERNAL_CRYSTAL_FREQUENCY_HZ: u32 = 12_000_000;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = rp2040_hal::clocks::init_clocks_and_plls(
        EXTERNAL_CRYSTAL_FREQUENCY_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Setup USB
    let force_vbus_detect_bit = true;
    let usb_bus = UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        force_vbus_detect_bit,
        &mut pac.RESETS,
    );

    let bus_allocator = UsbBusAllocator::new(usb_bus);

    let poll_ms = 10;
    let mut hid_endpoint = HIDClass::new_with_settings(
        &bus_allocator,
        KeyboardReport::desc(),
        poll_ms,
        HidClassSettings {
            subclass: HidSubClass::NoSubClass,
            protocol: HidProtocol::Keyboard,
            config: ProtocolModeConfig::ForceReport,
            // locale: HidCountryCode::NotSupported,
            locale: HidCountryCode::US,
        },
    );

    // https://github.com/obdev/v-usb/blob/7a28fdc685952412dad2b8842429127bc1cf9fa7/usbdrv/USB-IDs-for-free.txt#L128
    let mut keyboard_usb_device = UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x16c0, 0x27db))
        .manufacturer("bschwind")
        .product("key ripper")
        .build();

    // Get the GPIO peripherals.
    let sio = rp2040_hal::Sio::new(pac.SIO);

    let pins =
        rp2040_hal::gpio::Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);

    // Set up keyboard matrix pins.
    let rows: &[&dyn InputPin<Error = Infallible>] = &[
        &pins.gpio26.into_pull_down_input(),
        &pins.gpio25.into_pull_down_input(),
        &pins.gpio27.into_pull_down_input(),
        &pins.gpio28.into_pull_down_input(),
        &pins.gpio15.into_pull_down_input(),
    ];

    let cols: &mut [&mut dyn OutputPin<Error = Infallible>] = &mut [
        &mut pins.gpio29.into_push_pull_output(),
        &mut pins.gpio16.into_push_pull_output(),
        &mut pins.gpio17.into_push_pull_output(),
        &mut pins.gpio18.into_push_pull_output(),
        &mut pins.gpio9.into_push_pull_output(),
        &mut pins.gpio10.into_push_pull_output(),
        &mut pins.gpio19.into_push_pull_output(),
        &mut pins.gpio11.into_push_pull_output(),
        &mut pins.gpio12.into_push_pull_output(),
        &mut pins.gpio13.into_push_pull_output(),
        &mut pins.gpio14.into_push_pull_output(),
        &mut pins.gpio20.into_push_pull_output(),
        &mut pins.gpio22.into_push_pull_output(),
        &mut pins.gpio23.into_push_pull_output(),
    ];

    // Timer-based resources.
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().0);

    let timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut scan_countdown = timer.count_down();

    scan_countdown.start(10.milliseconds());

    // Main keyboard polling loop.
    loop {
        keyboard_usb_device.poll(&mut [&mut hid_endpoint]);

        if scan_countdown.wait().is_ok() {
            // Scan the keys and send a report.
            let matrix = scan_keys(rows, cols, &mut delay);

            let keycodes = if matrix[12][1] {
                // The equals sign.
                [0x2E, 0, 0, 0, 0, 0]
            } else {
                [0, 0, 0, 0, 0, 0]
            };

            let report = KeyboardReport { modifier: 0, reserved: 0, leds: 0, keycodes };

            hid_endpoint.push_input(&report).ok();
        }

        hid_endpoint.pull_raw_output(&mut [0; 64]).ok();
    }
}

fn scan_keys(
    rows: &[&dyn InputPin<Error = Infallible>],
    columns: &mut [&mut dyn embedded_hal::digital::v2::OutputPin<Error = Infallible>],
    delay: &mut Delay,
) -> [[bool; 6]; 14] {
    let mut matrix = [[false; 6]; 14];

    for (gpio_col, matrix_col) in columns.iter_mut().zip(matrix.iter_mut()) {
        gpio_col.set_high().unwrap();
        delay.delay_us(10);

        for (gpio_row, matrix_row) in rows.iter().zip(matrix_col.iter_mut()) {
            *matrix_row = gpio_row.is_high().unwrap();
        }

        gpio_col.set_low().unwrap();
        delay.delay_us(10);
    }

    matrix
}
