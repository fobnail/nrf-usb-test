#![no_std]
#![no_main]

#[macro_use]
extern crate rtt_target;
extern crate nrf52840_hal as hal;

use cortex_m_rt::entry;
use hal::clocks::Clocks;
use hal::usbd::{UsbPeripheral, Usbd};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_ethernet::EthernetDriver;
use usbd_serial::SerialPort;

#[cfg(target_os = "none")]
mod panic;

const EEM_BUFFER_SIZE: u16 = 1500 * 2;

fn main() -> ! {
    rtt_target::rtt_init_print!();
    rprintln!("Hello");

    let mut eth_rx_buf = [0u8; EEM_BUFFER_SIZE as usize];
    let mut eth_tx_buf = [0u8; EEM_BUFFER_SIZE as usize];

    let periph = nrf52840_hal::pac::Peripherals::take().unwrap();
    let clocks = Clocks::new(periph.CLOCK);
    let clocks = clocks.enable_ext_hfosc();

    let usb_bus = Usbd::new(UsbPeripheral::new(periph.USBD, &clocks));
    let mut serial = SerialPort::new(&usb_bus);
    let mut eth = EthernetDriver::new(&usb_bus, 64, &mut eth_rx_buf, &mut eth_tx_buf);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1234, 0x4231))
        .manufacturer("Test")
        .product("Test")
        .serial_number("001122334455667788")
        .device_class(0)
        .max_packet_size_0(64) // (makes control transfers 8x faster)
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut serial, &mut eth]) {
            continue;
        }

        let mut buf = [0u8; 64];

        let _ = serial.read(&mut buf);
        eth.read_packet(|p| rprintln!("Got EEM packet len {}", p.len()));
    }
}

#[entry]
fn start() -> ! {
    main()
}
