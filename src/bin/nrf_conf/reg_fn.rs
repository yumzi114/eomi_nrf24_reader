use defmt::info;
use embassy_stm32::{gpio::Output, spi};
use embassy_time::{Duration, Timer};
use embedded_hal_1::spi::SpiDevice;

use super::{FLUSH_RX, R_REGISTER, SPI_DRIVER, W_REGISTER};




pub async fn read_register(
    spi: &mut spi::Spi<'_, embassy_stm32::mode::Async>,
    nss: &mut Output<'_>,
    reg: u8,
    len: usize,
) {
    let mut tx_buffer = [0u8; 32];
    let mut rx_buffer = [0u8; 32];

    // R_REGISTER 명령어 + 레지스터 주소
    tx_buffer[0] = R_REGISTER | (reg & 0x1F);

    for i in 1..=len {
        tx_buffer[i] = 0xFF; // dummy write
    }

    nss.set_low();
    spi.transfer(&mut rx_buffer[..=len], &mut tx_buffer[..=len]).await.unwrap();
    nss.set_high();
    info!("READ {:02X} => {:?}", reg, &rx_buffer[1..=len]);
    info!("STATUS: {:08b}, CONFIG: {:08b}", rx_buffer[0], rx_buffer[1]);
}


pub async fn write_register(
    // spi: &mut spi::Spi<'_, embassy_stm32::mode::Async>, 
    spi: &mut SPI_DRIVER, 
    // nss: &mut Output<'_>, 
    reg: u8, 
    value: &[u8])
{
    let len = 1 + value.len();
    // const asd: /* Type */ = value.len()+1;
    // let asd: usize = value.len()+1;
    let mut tx_buffer = [0_u8; 32];
    let mut rx_buffer = [0_u8; 32];
    // let mut buffer = Vec::<_, 12>::new();
    // buffer.push(0b0010_0000 | (reg & 0x1F)).unwrap();

    tx_buffer[0] = W_REGISTER | (reg & 0x1F);
    tx_buffer[1..(1 + value.len())].copy_from_slice(value);
    // buffer[0] = 0b0010_0000 | (reg & 0x1F); 
    // buffer[1..].copy_from_slice(value);
    // nss.set_low();
    spi.transfer(&mut rx_buffer[..len], &mut tx_buffer[..len]).unwrap();
    // spi.transfer(&mut rx_buffer[..len], &mut tx_buffer[..len]).await.unwrap();
    // nss.set_high();
    // Timer::after(Duration::from_millis(10)).await;

}

pub async fn flush_rx(
    spi: &mut SPI_DRIVER,
    // nss: &mut Output<'_>,
) {
    let mut buf = [FLUSH_RX];
    let mut rx_buf = [0x00];
    // nss.set_low();
    spi.transfer(&mut rx_buf, &mut buf).unwrap();
    // nss.set_high();
}