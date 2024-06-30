// Simple keyboard firmware. Inspired by the RustyKeys project:
// https://github.com/KOBA789/rusty-keys/blob/main/firmware/keyboard/src/bin/simple.rs

#![no_main]
#![no_std]

mod debounce;
mod hid_class;
mod hid_descriptor;
mod key_codes;
mod key_mapping;
mod key_scan;

use crate::{
    hid_class::HidClass,
    key_scan::{KeyboardReport, TRANSPOSED_NORMAL_LAYER_MAPPING},
};
use core::{cell::RefCell, convert::Infallible};
use cortex_m::prelude::_embedded_hal_timer_CountDown;
use critical_section::Mutex;
use defmt::{error, info, warn};
use defmt_rtt as _;
use embedded_hal::digital::{InputPin, OutputPin};
use fugit::ExtU32;
use panic_probe as _;
use rp2040_hal::{
    pac::{self, interrupt},
    usb::{self, UsbBus},
    Clock, Watchdog,
};
use usb_device::{bus::UsbBusAllocator, device::UsbDeviceBuilder, prelude::*};
use usbd_hid::hid_class::{
    HIDClass, HidClassSettings, HidCountryCode, HidProtocol, HidSubClass, ProtocolModeConfig,
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
static USB_HID: Mutex<RefCell<Option<HIDClass<usb::UsbBus>>>> = Mutex::new(RefCell::new(None));

/// The USB Human Interface Device Driver (shared with the interrupt).
static USB_HID_CLASS: Mutex<RefCell<Option<HidClass<usb::UsbBus>>>> =
    Mutex::new(RefCell::new(None));

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
    let mut rows: [&mut dyn InputPin<Error = Infallible>; NUM_ROWS] = [
        &mut pins.gpio26.into_pull_down_input(),
        &mut pins.gpio25.into_pull_down_input(),
        &mut pins.gpio27.into_pull_down_input(),
        &mut pins.gpio28.into_pull_down_input(),
        &mut pins.gpio15.into_pull_down_input(),
        &mut pins.gpio24.into_pull_down_input(),
    ];

    let mut cols: [&mut dyn OutputPin<Error = Infallible>; NUM_COLS] = [
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
    let timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let mut modifier_mask = [[false; NUM_ROWS]; NUM_COLS];
    for (col, mapping_col) in modifier_mask.iter_mut().zip(TRANSPOSED_NORMAL_LAYER_MAPPING) {
        for (key, mapping_key) in col.iter_mut().zip(mapping_col) {
            *key = mapping_key.is_modifier();
        }
    }

    // Create a global debounce state to prevent unintended rapid key double-presses.
    let mut debounce: Debounce<NUM_ROWS, NUM_COLS> = Debounce::new(DEBOUNCE_TICKS, modifier_mask);

    // Do an initial scan of the keys so that we immediately have something to report to the host when asked.
    let scan = KeyScan::scan(&mut rows, &mut cols, &mut delay, &mut debounce);

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
    let bus_allocator_ref = unsafe {
        // Note (safety): This is safe as interrupts haven't been started yet
        USB_BUS = Some(bus_allocator);
        // We are promising to the compiler not to take mutable access to this global
        // variable while this reference exists!
        USB_BUS.as_ref().unwrap()
    };

    let hid_endpoint = HIDClass::new_with_settings(
        bus_allocator_ref,
        hid_descriptor::KEYBOARD_REPORT_DESCRIPTOR,
        USB_POLL_RATE_MS,
        HidClassSettings {
            subclass: HidSubClass::NoSubClass,
            protocol: HidProtocol::Keyboard,
            config: ProtocolModeConfig::ForceReport,
            locale: HidCountryCode::US,
        },
    );

    let hid_class = HidClass::new(bus_allocator_ref);

    // https://github.com/obdev/v-usb/blob/7a28fdc685952412dad2b8842429127bc1cf9fa7/usbdrv/USB-IDs-for-free.txt#L128
    let keyboard_usb_device = UsbDeviceBuilder::new(bus_allocator_ref, UsbVidPid(0x16c0, 0x27db))
        .supports_remote_wakeup(true)
        .strings(&[StringDescriptors::default().manufacturer("bschwind").product("key ripper")])
        .unwrap()
        .build();

    unsafe {
        // Note (safety): This is safe as interrupts haven't been started yet
        critical_section::with(|cs| {
            USB_HID.replace(cs, Some(hid_endpoint));
            USB_HID_CLASS.replace(cs, Some(hid_class));
        });

        USB_DEVICE = Some(keyboard_usb_device);
    }
    info!("Enabling USB interrupt handler");
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::USBCTRL_IRQ);
    }
    info!("Entering main loop");

    let mut tick_count_down = timer.count_down();
    tick_count_down.start(1.millis());

    let mut last_report: KeyboardReport = scan.into();

    loop {
        if tick_count_down.wait().is_ok() {
            let scan = KeyScan::scan(&mut rows, &mut cols, &mut delay, &mut debounce);
            let report: KeyboardReport = scan.into();

            if report != last_report {
                critical_section::with(|cs| {
                    let mut usb_hid = USB_HID.borrow_ref_mut(cs);
                    let usb_hid = usb_hid.as_mut().unwrap();

                    let mut hid_class = USB_HID_CLASS.borrow_ref_mut(cs);
                    let hid_class = hid_class.as_mut().unwrap();

                    if let Err(err) = usb_hid.push_raw_input(&report.as_raw_input()) {
                        match err {
                            UsbError::WouldBlock => warn!("UsbError::WouldBlock"),
                            UsbError::ParseError => error!("UsbError::ParseError"),
                            UsbError::BufferOverflow => error!("UsbError::BufferOverflow"),
                            UsbError::EndpointOverflow => error!("UsbError::EndpointOverflow"),
                            UsbError::EndpointMemoryOverflow => {
                                error!("UsbError::EndpointMemoryOverflow")
                            },
                            UsbError::InvalidEndpoint => error!("UsbError::InvalidEndpoint"),
                            UsbError::Unsupported => error!("UsbError::Unsupported"),
                            UsbError::InvalidState => error!("UsbError::InvalidState"),
                        }
                    }

                    let _ = hid_class.write_raw_report(&report.as_raw_input());
                });

                last_report = report;
            }
        }
    }
}

/// Handle USB interrupts
#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    let usb_dev = USB_DEVICE.as_mut().unwrap();

    critical_section::with(|cs| {
        let mut usb_hid = USB_HID.borrow_ref_mut(cs);
        let usb_hid = usb_hid.as_mut().unwrap();

        let mut hid_class = USB_HID_CLASS.borrow_ref_mut(cs);
        let hid_class = hid_class.as_mut().unwrap();

        usb_dev.poll(&mut [usb_hid, hid_class]);

        // macOS doesn't like it when you don't pull this, apparently.
        // TODO: maybe even parse something here
        usb_hid.pull_raw_output(&mut [0; 64]).ok();
    });
}
