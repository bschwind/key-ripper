// Simple keyboard firmware. Inspired by the RustyKeys project:
// https://github.com/KOBA789/rusty-keys/blob/main/firmware/keyboard/src/bin/simple.rs

#![no_main]
#![no_std]

mod debounce;
mod hid_descriptor;
mod key_codes;
mod key_mapping;
mod key_scan;

use core::{cell::RefCell, convert::Infallible};
use critical_section::Mutex;
use defmt::{error, info, warn};
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_probe as _;
use rp2040_hal::{
    pac::{self, interrupt},
    usb::{self, UsbBus},
    Clock, Watchdog,
};
use usb_device::{bus::UsbBusAllocator, device::UsbDeviceBuilder, prelude::*};
use usbd_hid::{
    descriptor::KeyboardReport,
    hid_class::{
        HIDClass, HidClassSettings, HidCountryCode, HidProtocol, HidSubClass, ProtocolModeConfig,
    },
};

use debounce::Debounce;
use key_scan::KeyScan;

/// The rate of polling of the keyboard itself in firmware.
const SCAN_LOOP_RATE_MS: u32 = 1;
/// The rate of USB interrupt polling the device will ask of the host.
const USB_POLL_RATE_MS: u8 = SCAN_LOOP_RATE_MS as u8;
/// The number of milliseconds to wait until a "key-off-then-key-on" in quick succession is allowed.
const DEBOUNCE_MS: u8 = 6;

const DEBOUNCE_TICKS: u8 = DEBOUNCE_MS / (SCAN_LOOP_RATE_MS as u8);

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const NUM_COLS: usize = 14;
const NUM_ROWS: usize = 6;

const EXTERNAL_CRYSTAL_FREQUENCY_HZ: u32 = 12_000_000;

/// The USB Device Driver (shared with the interrupt).
static mut USB_DEVICE: Option<UsbDevice<usb::UsbBus>> = None;

/// The USB Bus Driver (shared with the interrupt).
static mut USB_BUS: Option<UsbBusAllocator<usb::UsbBus>> = None;

/// The USB Human Interface Device Driver (shared with the interrupt).
static mut USB_HID: Option<HIDClass<usb::UsbBus>> = None;

/// The latest keyboard report for responding to USB interrupts.
static KEYBOARD_REPORT: Mutex<RefCell<KeyboardReport>> = Mutex::new(RefCell::new(KeyboardReport {
    modifier: 0,
    reserved: 0,
    leds: 0,
    keycodes: [0u8; 6],
}));

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

    // Initialize a delay for accurate sleeping.
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let mut modifier_mask = [[false; NUM_ROWS]; NUM_COLS];
    for (col, mapping_col) in modifier_mask.iter_mut().zip(key_mapping::NORMAL_LAYER_MAPPING) {
        for (key, mapping_key) in col.iter_mut().zip(mapping_col) {
            *key = mapping_key.is_modifier();
        }
    }

    // Create a global debounce state to prevent unintended rapid key double-presses.
    let mut debounce: Debounce<NUM_ROWS, NUM_COLS> = Debounce::new(DEBOUNCE_TICKS, modifier_mask);

    // Do an initial scan of the keys so that we immediately have something to report to the host when asked.
    let scan = KeyScan::scan(rows, cols, &mut delay, &mut debounce);
    critical_section::with(|cs| {
        KEYBOARD_REPORT.replace(cs, scan.into());
    });

    // If the Escape key is pressed during power-on, we should go into bootloader mode.
    if scan[0][0] {
        let gpio_activity_pin_mask = 0;
        let disable_interface_mask = 0;
        info!("Escape key detected on boot, going into bootloader mode.");
        rp2040_hal::rom_data::reset_to_usb_boot(gpio_activity_pin_mask, disable_interface_mask);
    }

    info!("Initializing USB");
    // Initialize USB
    let force_vbus_detect_bit = true;
    let usb_bus = UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        force_vbus_detect_bit,
        &mut pac.RESETS,
    );
    let bus_allocator = UsbBusAllocator::new(usb_bus);
    let bus_ref = unsafe {
        // Note (safety): This is safe as interrupts haven't been started yet
        USB_BUS = Some(bus_allocator);
        // We are promising to the compiler not to take mutable access to this global
        // variable while this reference exists!
        USB_BUS.as_ref().unwrap()
    };

    let hid_endpoint = HIDClass::new_with_settings(
        bus_ref,
        hid_descriptor::KEYBOARD_REPORT_DESCRIPTOR,
        USB_POLL_RATE_MS,
        HidClassSettings {
            subclass: HidSubClass::NoSubClass,
            protocol: HidProtocol::Keyboard,
            config: ProtocolModeConfig::ForceReport,
            locale: HidCountryCode::US,
        },
    );

    // https://github.com/obdev/v-usb/blob/7a28fdc685952412dad2b8842429127bc1cf9fa7/usbdrv/USB-IDs-for-free.txt#L128
    let keyboard_usb_device = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x16c0, 0x27db))
        .manufacturer("bschwind")
        .product("key ripper")
        .build();
    unsafe {
        // Note (safety): This is safe as interrupts haven't been started yet
        USB_HID = Some(hid_endpoint);
        USB_DEVICE = Some(keyboard_usb_device);
    }
    info!("Enabling USB interrupt handler");
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::USBCTRL_IRQ);
    }
    info!("Entering main loop");
    loop {
        let scan = KeyScan::scan(rows, cols, &mut delay, &mut debounce);
        critical_section::with(|cs| {
            KEYBOARD_REPORT.replace(cs, scan.into());
        });
        delay.delay_ms(SCAN_LOOP_RATE_MS);
    }
}

/// Handle USB interrupts, used by the host to "poll" the keyboard for new inputs.
#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    let usb_dev = USB_DEVICE.as_mut().unwrap();
    let usb_hid = USB_HID.as_mut().unwrap();

    usb_dev.poll(&mut [usb_hid]);

    let report = critical_section::with(|cs| *KEYBOARD_REPORT.borrow_ref(cs));
    if let Err(err) = usb_hid.push_input(&report) {
        match err {
            UsbError::WouldBlock => warn!("UsbError::WouldBlock"),
            UsbError::ParseError => error!("UsbError::ParseError"),
            UsbError::BufferOverflow => error!("UsbError::BufferOverflow"),
            UsbError::EndpointOverflow => error!("UsbError::EndpointOverflow"),
            UsbError::EndpointMemoryOverflow => error!("UsbError::EndpointMemoryOverflow"),
            UsbError::InvalidEndpoint => error!("UsbError::InvalidEndpoint"),
            UsbError::Unsupported => error!("UsbError::Unsupported"),
            UsbError::InvalidState => error!("UsbError::InvalidState"),
        }
    }

    // macOS doesn't like it when you don't pull this, apparently.
    // TODO: maybe even parse something here
    usb_hid.pull_raw_output(&mut [0; 64]).ok();
}
