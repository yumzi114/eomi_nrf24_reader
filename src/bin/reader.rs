#![no_std]
#![no_main]
use cortex_m_rt::entry;
use defmt::*;
use display_fn::{i2c_display};
use embassy_stm32::{bind_interrupts, gpio::{Input, Level, Output, Pull, Speed}, i2c::{self, I2c}, peripherals, spi, time::{mhz, Hertz}};
use embedded_hal_1::spi::SpiDevice;
use embedded_hal_bus::spi::ExclusiveDevice;
use nrf_conf::{module_init, reg_fn::{flush_rx, write_register}, RX_DR, R_REGISTER, R_RX_PAYLOAD, STATUS};
use {defmt_rtt as _, panic_probe as _};
use embassy_executor::{Executor, Spawner};
// use embedded_hal::spi::SpiBus;
use embassy_time::{ Duration, Timer};
use embassy_time::Delay;
use core::sync::atomic::{AtomicI16, AtomicU32, Ordering};
use embassy_stm32::exti::ExtiInput;
mod nrf_conf;
mod display_fn;

static COUNT: AtomicU32 = AtomicU32::new(0);
static L_X: AtomicI16 = AtomicI16::new(0);
static L_Y: AtomicI16 = AtomicI16::new(0);
static R_Y: AtomicI16 = AtomicI16::new(0);
static R_X: AtomicI16 = AtomicI16::new(0);
bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});
#[embassy_executor::main]
async fn main(spawner: Spawner)  {
    // info!("Hello World!");
    let config={
        use embassy_stm32::rcc::*;
        let mut config = embassy_stm32::Config::default();
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(25),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV25,
            mul: PllMul::MUL192,
            divp: Some(PllPDiv::DIV2),
            divq: Some(PllQDiv::DIV4),
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P;

        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
        config.rcc.plli2s = Some(Pll {
            prediv: PllPreDiv::DIV25,
            mul: PllMul::MUL384,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV5),
        });
        config.enable_debug_during_sleep = true;

        config
    };

    let p = embassy_stm32::init(config); 
    
    //RF SPI CONFIG
    let mut delay = Delay;
    let mut rf_spi_config = spi::Config::default();
    rf_spi_config.mode = embassy_stm32::spi::MODE_0;
    rf_spi_config.frequency = mhz(10);
    // let delay = Delay;
    let mut irq: ExtiInput = ExtiInput::new(p.PA9, p.EXTI9, Pull::None);
    // let mut irq = Input::new(p.PA9, Pull::None);
    let mut ce: Output = Output::new(p.PA8, Level::High, Speed::Medium);
    ce.set_low();
    let mut nss = Output::new(p.PB12, Level::High, Speed::Medium);
    
    let rf_spi= spi::Spi::new(p.SPI2, p.PB13, p.PB15, p.PB14, p.DMA1_CH4,p.DMA1_CH3,rf_spi_config);
    let mut spi_device = ExclusiveDevice::new_no_delay(rf_spi, nss).unwrap();
   
    let i2c: I2c<embassy_stm32::mode::Async> = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB7,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH0,
        Hertz(480_000),
        Default::default(),
    );
    Timer::after(Duration::from_millis(5)).await;
    module_init(&mut ce,&mut spi_device).await;
    spawner.spawn(i2c_display( i2c)).ok();
    loop{
        irq.wait_for_falling_edge().await;
        let mut status = [0u8; 2];
        spi_device.transfer(&mut status, &mut [R_REGISTER | STATUS, 0xFF]).unwrap();

        if status[1] & RX_DR != 0 {
            // 데이터 수신
            // let mut tx_buf = [R_RX_PAYLOAD];
            let mut rx_buf = [0u8; 9]; // 1 바이트 명령어 + 5바이트 데이터

            // nss.set_low();
            spi_device.transfer(&mut rx_buf, &mut [R_RX_PAYLOAD, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]).unwrap();
            // nss.set_high();


            COUNT.fetch_add(1, Ordering::SeqCst);
            R_X.store(i16::from_be_bytes([rx_buf[3],rx_buf[4]]), Ordering::SeqCst);
            R_Y.store(i16::from_be_bytes([rx_buf[1],rx_buf[2]]), Ordering::SeqCst);
            L_X.store(i16::from_be_bytes([rx_buf[7],rx_buf[8]]), Ordering::SeqCst);
            L_Y.store(i16::from_be_bytes([rx_buf[5],rx_buf[6]]), Ordering::SeqCst);
            write_register(&mut spi_device,  STATUS, &[RX_DR]).await;
            flush_rx(&mut spi_device ).await;
            
        }
    }
}
