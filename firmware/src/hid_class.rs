use crate::hid_descriptor::KEYBOARD_REPORT_DESCRIPTOR;
use core::marker::PhantomData;
use usb_device::{
    class_prelude::{
        BosWriter, ControlIn, ControlOut, DescriptorWriter, EndpointAddress, EndpointIn,
        InterfaceNumber, StringIndex, UsbBus, UsbBusAllocator, UsbClass,
    },
    control::{Request, RequestType},
    LangID, Result,
};

const USB_CLASS_HID: u8 = 0x03;

const DESCRIPTOR_LEN_BYTES: [u8; 2] = (KEYBOARD_REPORT_DESCRIPTOR.len() as u16).to_le_bytes();

// This is usually prepended with the length (including the byte for the length itself),
// and the descriptor type, so 2 extra bytes.
const HID_DESCRIPTOR: [u8; 7] = [
    0x11, // bcdHID - 1.11 - LSB first
    0x01, // bcdHID - 1.11 - LSB first
    0x00, // bCountryCode - 0 = Not supported/specified
    1,    // bNumDescriptors - Number of HID class descriptors to follow
    // bDescriptorType
    //   * 0x21      - HID
    //   * 0x22      - Report
    //   * 0x23      - Physical descriptor
    //   * 0x24-0x2F - Reserved
    0x22,                    // bDescriptorType - Report
    DESCRIPTOR_LEN_BYTES[0], // wDescriptorLength - LSB first
    DESCRIPTOR_LEN_BYTES[1], // wDescriptorLength - LSB first
];

#[derive(Debug, Default, Copy, Clone)]
pub struct LedState {
    num_lock: bool,
    caps_lock: bool,
    scroll_lock: bool,
    compose: bool,
    kana: bool,
}

impl From<u8> for LedState {
    fn from(byte: u8) -> Self {
        Self {
            num_lock: byte & 1 == 1,
            caps_lock: (byte >> 1) & 1 == 1,
            scroll_lock: (byte >> 2) & 1 == 1,
            compose: (byte >> 3) & 1 == 1,
            kana: (byte >> 4) & 1 == 1,
        }
    }
}

// A HID device is composed of the following endpoints:
// * A pair of control IN and OUT endpoints called the default endpoint
//   (these are handled by usb-device?)
// * An interrupt IN endpoint
// * An optional interrupt OUT endpoint
pub struct HidClass<'a, B: UsbBus> {
    usb_interface: InterfaceNumber,

    // The Interrupt pipe are used for:
    // * Receiving asynchronous (unrequested) data from the device.
    // * Transmitting low latency data to the device.
    in_endpoint: EndpointIn<'a, B>,

    // The Interrupt Out pipe is optional. If a device declares an Interrupt Out endpoint
    // then Output reports are transmitted by the host to the device through the Interrupt
    // Out endpoint. If no Interrupt Out endpoint is declared then Output reports are
    // transmitted to a device through the Control endpoint, using Set_Report(Output)
    // requests.
    // An Interrupt Out pipe is optional and requires an additional Endpoint descriptor
    // if declared.
    // out_endpoint: EndpointOut<'a, B>,
    _bus: PhantomData<B>,

    last_report: [u8; 8],
    led_state: LedState,
}

impl<'a, B: UsbBus> HidClass<'a, B> {
    pub fn new(bus_allocator: &'a UsbBusAllocator<B>) -> Self {
        let usb_interface = bus_allocator.interface();

        let max_packet_size = 8;

        // Poll every 1 ms. Device must be in USB Full-Speed for this to work, along with USB 1.1 or greater.
        let poll_interval = 1;
        let in_endpoint = bus_allocator.interrupt(max_packet_size, poll_interval);

        let last_report = [0; 8];

        Self {
            usb_interface,
            in_endpoint,
            _bus: PhantomData {},
            last_report,
            led_state: LedState::default(),
        }
    }

    pub fn write_raw_report(&mut self, data: [u8; 8]) -> Result<usize> {
        self.last_report = data;
        self.in_endpoint.write(&data)
    }

    pub fn led_state(&self) -> LedState {
        self.led_state
    }
}

impl<B: UsbBus> UsbClass<B> for HidClass<'_, B> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
        // When a Get_Descriptor(Configuration) request is issued, it
        // returns the Configuration descriptor, all Interface descriptors, all Endpoint
        // descriptors, and the HID descriptor for each interface.
        // That is, the order shall be:
        //   * Configuration descriptor (handled by usb-device crate)
        //   * Interface descriptor (specifying HID Class)
        //   * HID descriptor (associated with above Interface)
        //   * Endpoint descriptor (for HID Interrupt In Endpoint)
        //   * Optional Endpoint descriptor (for HID Interrupt Out Endpoint)

        // The bDeviceClass and bDeviceSubClass fields in the Device Descriptor
        // should not be used to identify a device as belonging to the HID class. Instead use
        // the bInterfaceClass and bInterfaceSubClass fields in the Interface descriptor.

        // The bInterfaceClass member of an Interface descriptor is always 3 for HID class devices.

        // The bInterfaceSubClass member declares whether a device supports a boot interface, otherwise it is 0.
        //     0     - No subclass
        //     1     - Boot interface subclass
        //     2-255 - Reserved

        // The bInterfaceProtocol member of an Interface descriptor only has meaning if the bInterfaceSubClass
        // member declares that the device supports a boot interface, otherwise it is 0
        //     0     - None
        //     1     - Keyboard
        //     2     - Mouse
        //     3-255 - Reserved

        // Write the interface descriptor
        writer.interface(
            self.usb_interface,
            USB_CLASS_HID,
            1, // Boot interface subclass
            1, // Keyboard
        )?;

        // Write the HID Descriptor
        writer.write(
            // Descriptor type
            // 0x21      - HID
            // 0x22      - Report
            // 0x23      - Physical Descriptor
            // 0x24-0x2F - Reserved
            0x21, // bDescriptorType
            &HID_DESCRIPTOR,
        )?;

        // Write the descriptor for the IN endpoint
        writer.endpoint(&self.in_endpoint)?;

        // Write the descriptor for the OUT endpoint, if we have one.

        Ok(())
    }

    fn get_bos_descriptors(&self, writer: &mut BosWriter) -> Result<()> {
        let _ = writer;
        Ok(())
    }

    fn get_string(&self, index: StringIndex, lang_id: LangID) -> Option<&str> {
        let _ = (index, lang_id);
        None
    }

    fn reset(&mut self) {}

    fn poll(&mut self) {}

    fn control_out(&mut self, xfer: ControlOut<B>) {
        const SET_REPORT_REQUEST: u8 = 0x09;

        let request = xfer.request();

        let interface = request.index;
        if interface != u8::from(self.usb_interface) as u16 {
            return;
        }

        if request.request == SET_REPORT_REQUEST {
            let data_len = xfer.data().len();

            if data_len > 0 {
                // The keyboard OUT report is 1 byte for the LED state.
                self.led_state = LedState::from(xfer.data()[0]);
            }

            xfer.accept().ok();
        }
    }

    // The Control pipe is used for:
    // * Receiving and responding to requests for USB control and class data.
    // * Transmitting data when polled by the HID class driver (using the Get_Report request).
    // * Receiving data from the host.
    fn control_in(&mut self, xfer: ControlIn<B>) {
        const GET_REPORT_REQUEST: u8 = 0x01;
        // const GET_IDLE_REQUEST: u8 = 0x02;
        // const GET_PROTOCOL_REQUEST: u8 = 0x03;
        // const SET_REPORT_REQUEST: u8 = 0x09;
        // const SET_IDLE_REQUEST: u8 = 0x0A;
        // const SET_PROTOCOL_REQUEST: u8 = 0x0B;

        let request = xfer.request();

        let interface = request.index;
        if interface != u8::from(self.usb_interface) as u16 {
            return;
        }

        match (request.request_type, request.request) {
            (RequestType::Standard, Request::GET_DESCRIPTOR) => {
                let [_descriptor_index, descriptor_type] = request.value.to_le_bytes();

                match descriptor_type {
                    // HID Descriptor Type
                    0x21 => {
                        let buf: [u8; HID_DESCRIPTOR.len() + 2] = [
                            // Length of buf inclusive of size prefix
                            HID_DESCRIPTOR.len() as u8 + 2,
                            0x21, // HID Descriptor type
                            HID_DESCRIPTOR[0],
                            HID_DESCRIPTOR[1],
                            HID_DESCRIPTOR[2],
                            HID_DESCRIPTOR[3],
                            HID_DESCRIPTOR[4],
                            HID_DESCRIPTOR[5],
                            HID_DESCRIPTOR[6],
                        ];

                        xfer.accept_with(&buf).ok();
                    },
                    // HID Report Descriptor Type
                    0x22 => {
                        xfer.accept_with_static(KEYBOARD_REPORT_DESCRIPTOR).ok();
                    },
                    _ => {},
                }
            },
            (RequestType::Class, GET_REPORT_REQUEST) => {
                const REPORT_TYPE_INPUT: u8 = 0x01;
                // const REPORT_TYPE_OUTPUT: u8 = 0x02;
                // const REPORT_TYPE_FEATURE: u8 = 0x03;

                let [_report_id, report_type] = request.value.to_le_bytes();

                if report_type == REPORT_TYPE_INPUT {
                    xfer.accept_with(&self.last_report).ok();
                }
            },
            _ => {},
        }
    }

    fn endpoint_setup(&mut self, addr: EndpointAddress) {
        let _ = addr;
    }

    fn endpoint_out(&mut self, addr: EndpointAddress) {
        let _ = addr;
    }

    fn endpoint_in_complete(&mut self, addr: EndpointAddress) {
        let _ = addr;
    }

    fn get_alt_setting(&mut self, interface: InterfaceNumber) -> Option<u8> {
        let _ = interface;
        None
    }

    fn set_alt_setting(&mut self, interface: InterfaceNumber, alternative: u8) -> bool {
        let _ = (interface, alternative);
        false
    }
}
