use crate::hid_descriptor::KEYBOARD_REPORT_DESCRIPTOR;
use core::marker::PhantomData;
use usb_device::{
    class_prelude::{
        BosWriter, ControlIn, ControlOut, DescriptorWriter, EndpointAddress, EndpointIn,
        InterfaceNumber, StringIndex, UsbBus, UsbBusAllocator, UsbClass,
    },
    LangID, Result,
};

const USB_CLASS_HID: u8 = 0x03;

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
}

impl<'a, B: UsbBus> HidClass<'a, B> {
    pub fn new(bus_allocator: &'a UsbBusAllocator<B>) -> Self {
        let usb_interface = bus_allocator.interface();

        let max_packet_size = 8;
        let poll_interval = 1; // Poll every 1 ms.
        let in_endpoint = bus_allocator.interrupt(max_packet_size, poll_interval);

        Self { usb_interface, in_endpoint, _bus: PhantomData {} }
    }
}

impl<B: UsbBus> UsbClass<B> for HidClass<'_, B> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
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

        writer.interface(
            self.usb_interface,
            USB_CLASS_HID,
            1, // Boot interface subclass
            1, // Keyboard
        )?;

        let descriptor_len = KEYBOARD_REPORT_DESCRIPTOR.len() as u16;
        let [descriptor_len_lsb, descriptor_len_msb] = descriptor_len.to_le_bytes();
        // let descriptor_len_lsb =

        // Write the HID Descriptor
        writer.write(
            // Descriptor type
            // 0x21      - HID
            // 0x22      - Report
            // 0x23      - Physical Descriptor
            // 0x24-0x2F - Reserved
            0x21, // bDescriptorType
            &[
                0x11, // bcdHID - 1.11 - LSB first
                0x01, // bcdHID - 1.11 - LSB first
                0x00, // bCountryCode - 0 = Not supported/specified
                1,    // bNumDescriptors - Number of HID class descriptors to follow
                // bDescriptorType
                //   * 0x21      - HID
                //   * 0x22      - Report
                //   * 0x23      - Physical descriptor
                //   * 0x24-0x2F - Reserved
                0x22,               // bDescriptorType - Report
                descriptor_len_lsb, // wDescriptorLength - LSB first
                descriptor_len_msb, // wDescriptorLength - LSB first
            ],
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
        let _ = xfer;
    }

    // The Control pipe is used for:
    // * Receiving and responding to requests for USB control and class data.
    // * Transmitting data when polled by the HID class driver (using the Get_Report request).
    // * Receiving data from the host.
    fn control_in(&mut self, xfer: ControlIn<B>) {
        let _ = xfer;
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
