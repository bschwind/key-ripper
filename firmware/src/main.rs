// Simple keyboard firmware. Inspired by the RustyKeys project:
// https://github.com/KOBA789/rusty-keys/blob/main/firmware/keyboard/src/bin/simple.rs

#![no_main]
#![no_std]

use core::convert::Infallible;
use cortex_m::delay::Delay;
use defmt::{error, info};
use defmt_rtt as _;
use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    timer::CountDown,
};
use embedded_time::duration::Extensions;
use keycodes::KeyCode;
// use panic_reset as _;
use panic_probe as _;
use rp2040_hal::{pac, usb::UsbBus, Clock, Watchdog};
use usb_device::{bus::UsbBusAllocator, device::UsbDeviceBuilder, prelude::UsbVidPid, UsbError};
use usbd_hid::{
    descriptor::KeyboardReport,
    hid_class::{
        HIDClass, HidClassSettings, HidCountryCode, HidProtocol, HidSubClass, ProtocolModeConfig,
    },
};

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

mod hid_descriptor;
mod keycodes;

const NUM_COLS: usize = 14;
const NUM_ROWS: usize = 6;

#[rustfmt::skip]
const MATRIX_MAPPING: [[KeyCode; NUM_ROWS]; NUM_COLS] = [
    [KeyCode::Escape, KeyCode::Tilde, KeyCode::Tab, KeyCode::CapsLock, KeyCode::LeftShift, KeyCode::Fn],
    [KeyCode::F1, KeyCode::Num1, KeyCode::Q, KeyCode::A, KeyCode::Empty, KeyCode::LeftCtrl],
    [KeyCode::F2, KeyCode::Num2, KeyCode::W, KeyCode::S, KeyCode::Z, KeyCode::LeftAlt],
    [KeyCode::F3, KeyCode::Num3, KeyCode::E, KeyCode::D, KeyCode::X, KeyCode::LeftCmd],
    [KeyCode::F4, KeyCode::Num4, KeyCode::R, KeyCode::F, KeyCode::C, KeyCode::Empty],
    [KeyCode::F5, KeyCode::Num5, KeyCode::T, KeyCode::G, KeyCode::V, KeyCode::Empty],
    [KeyCode::Empty, KeyCode::Num6, KeyCode::Y, KeyCode::H, KeyCode::B, KeyCode::Space],
    [KeyCode::F6, KeyCode::Num7, KeyCode::U, KeyCode::J, KeyCode::N, KeyCode::Empty],
    [KeyCode::F7, KeyCode::Num8, KeyCode::I, KeyCode::K, KeyCode::M, KeyCode::Empty],
    [KeyCode::F8, KeyCode::Num9, KeyCode::O, KeyCode::L, KeyCode::Comma, KeyCode::Empty],
    [KeyCode::F9, KeyCode::Num0, KeyCode::P, KeyCode::Semicolon, KeyCode::Period, KeyCode::RightCmd],
    [KeyCode::F10, KeyCode::Minus, KeyCode::LeftSquareBracket, KeyCode::SingleQuote, KeyCode::ForwardSlash, KeyCode::Left],
    [KeyCode::F11, KeyCode::Equals, KeyCode::RightSquareBracket, KeyCode::Enter, KeyCode::Up, KeyCode::Down],
    [KeyCode::F12, KeyCode::Backspace, KeyCode::BackSlash, KeyCode::Empty, KeyCode::Empty, KeyCode::Right],
];

const EXTERNAL_CRYSTAL_FREQUENCY_HZ: u32 = 12_000_000;

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Start of main()");
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

    let poll_ms = 8;
    let mut hid_endpoint = HIDClass::new_with_settings(
        &bus_allocator,
        hid_descriptor::KEYBOARD_REPORT_DESCRIPTOR,
        poll_ms,
        HidClassSettings {
            subclass: HidSubClass::NoSubClass,
            protocol: HidProtocol::Keyboard,
            config: ProtocolModeConfig::ForceReport,
            // locale: HidCountryCode::NotSupported,
            locale: HidCountryCode::US,
        },
    );

    info!("USB initialized");

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
        &pins.gpio24.into_pull_down_input(),
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

    // Start on a 500ms countdown so the USB endpoint writes don't block.
    scan_countdown.start(500.milliseconds());

    info!("Start main loop");

    // Main keyboard polling loop.
    loop {
        keyboard_usb_device.poll(&mut [&mut hid_endpoint]);

        if scan_countdown.wait().is_ok() {
            // Scan the keys and send a report.
            let matrix = scan_keys(rows, cols, &mut delay);
            let report = report_from_matrix(&matrix);

            match hid_endpoint.push_input(&report) {
                Ok(_) => {
                    scan_countdown.start(8.milliseconds());
                },
                Err(err) => match err {
                    UsbError::WouldBlock => {
                        info!("UsbError::WouldBlock");
                        scan_countdown.start(100.milliseconds());
                    },
                    UsbError::ParseError => error!("UsbError::ParseError"),
                    UsbError::BufferOverflow => error!("UsbError::BufferOverflow"),
                    UsbError::EndpointOverflow => error!("UsbError::EndpointOverflow"),
                    UsbError::EndpointMemoryOverflow => error!("UsbError::EndpointMemoryOverflow"),
                    UsbError::InvalidEndpoint => error!("UsbError::InvalidEndpoint"),
                    UsbError::Unsupported => error!("UsbError::Unsupported"),
                    UsbError::InvalidState => error!("UsbError::InvalidState"),
                },
            }
        }

        hid_endpoint.pull_raw_output(&mut [0; 64]).ok();
    }
}

fn scan_keys(
    rows: &[&dyn InputPin<Error = Infallible>],
    columns: &mut [&mut dyn embedded_hal::digital::v2::OutputPin<Error = Infallible>],
    delay: &mut Delay,
) -> [[bool; NUM_ROWS]; NUM_COLS] {
    let mut matrix = [[false; NUM_ROWS]; NUM_COLS];

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

fn report_from_matrix(matrix: &[[bool; NUM_ROWS]; NUM_COLS]) -> KeyboardReport {
    let mut keycodes = [0u8; 6];
    let mut keycode_index = 0;
    let mut modifier = 0;

    let mut push_keycode = |key| {
        if keycode_index < keycodes.len() {
            keycodes[keycode_index] = key;
            keycode_index += 1;
        }
    };

    for (matrix_column, mapping_column) in matrix.iter().zip(MATRIX_MAPPING) {
        for (key_pressed, mapping_row) in matrix_column.iter().zip(mapping_column) {
            if *key_pressed {
                if let Some(bitmask) = mapping_row.modifier_bitmask() {
                    modifier |= bitmask;
                } else {
                    push_keycode(mapping_row as u8);
                }
            }
        }
    }

    KeyboardReport { modifier, reserved: 0, leds: 0, keycodes }
}
