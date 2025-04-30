use core::{ops::Add, sync::atomic::Ordering};

use cortex_m::asm::nop;
use defmt::info;
use embassy_stm32::{exti::ExtiInput, gpio::Output, mode::Async, spi};
use embassy_time::{Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use reg_fn::{flush_rx, read_register, write_register};

use crate::COUNT;
type SPI_DRIVER =ExclusiveDevice<spi::Spi<'static, Async>, Output<'static>, embedded_hal_bus::spi::NoDelay>;
pub mod reg_fn;


pub const R_REGISTER: u8     = 0x00;
const W_REGISTER: u8     = 0x20;
const W_TX_PAYLOAD: u8   = 0xA0;
const FLUSH_TX: u8       = 0xE1;
pub const STATUS: u8         = 0x07;
const CONFIG: u8         = 0x00;
const EN_AA: u8          = 0x01;
const SETUP_RETR: u8     = 0x04;
const RF_CH: u8          = 0x05;
const RF_SETUP: u8       = 0x06;
const TX_ADDR: u8        = 0x10;
const FIFO_STATUS: u8    = 0x17;


const EN_RXADDR: u8    = 0x02;
const RX_ADDR_P0: u8   = 0x0A;
const RX_PW_P0: u8     = 0x11;
pub const R_RX_PAYLOAD: u8 = 0x61;

const FLUSH_RX: u8 = 0xE2;
// STATUS bits
const TX_DS: u8 = 1 << 5;
const MAX_RT: u8 = 1 << 4;
pub const RX_DR: u8 = 1 << 6;


pub async fn module_init(
    ce: &mut Output<'_>,
    spi: &mut SPI_DRIVER,
){
    let mut status = [0u8; 2];
    let addr: [u8; 5] = [0xE7, 0xE7, 0xE7, 0xE7, 0xE7];
    ce.set_low();
    write_register(spi, STATUS, &[0x70]).await;
    // 수신 모드 설정 (PWR_UP=1, PRIM_RX=1)
    write_register(spi, CONFIG, &[0b0000_1011]).await;
    write_register(spi,  EN_AA, &[0x01]).await;

    // 파이프 0 활성화
    write_register( spi,   EN_RXADDR, &[0x01]).await;

    // 채널 설정
    write_register( spi,   RF_CH, &[0x02]).await;

    // 데이터 속도 및 출력 세기 설정
    // 최장거리 통신	0x1E (250Kbps, 0dBm)
    //빠른 속도 통신	0x2E (2Mbps, 0dBm)
    //기본 안정성	0x0E (1Mbps, 0dBm)
    //E01-2G4M27D 셋팅 고출력시
    // write_register( spi,  RF_SETUP, &[0x0F]).await;
    write_register( spi,  RF_SETUP, &[0b0000_1110]).await;
    // 수신 주소 설정
    write_register( spi, RX_ADDR_P0, &addr).await;

    // 페이로드 길이 설정 (5바이트 예시)
    write_register( spi,   RX_PW_P0, &[0x08]).await;
    // 수신 버퍼 클리어
    flush_rx(spi).await;
    write_register(spi,  STATUS, &[RX_DR]).await;
    write_register(spi,  STATUS, &[0b0111_0000]).await; 
    Timer::after(Duration::from_millis(150)).await;
    ce.set_high();
    // Timer::after(Duration::from_millis(150)).await;
}

