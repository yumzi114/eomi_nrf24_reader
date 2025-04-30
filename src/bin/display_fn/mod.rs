use embassy_stm32::{i2c::I2c, mode::Async};
use embassy_time::{ Delay, Duration, Timer};
use embedded_graphics::{mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder}, pixelcolor::{BinaryColor, Rgb666}, prelude::{Point, RgbColor as _, Size}, primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle}, text::{Baseline, Text}};
use oled_async::{builder::Builder, mode::GraphicsMode, prelude::DisplayRotation};
use embedded_graphics::prelude::Primitive;
use embedded_graphics::Drawable;
use heapless::String;
use core::{fmt::Write, sync::atomic::Ordering};
use crate::{COUNT, L_X, L_Y, R_X, R_Y};






#[embassy_executor::task]
pub async fn i2c_display(
    i2c:I2c<'static, Async>,
) {
    let mut count_flag=0_u32;
    let mut stick_flag=[0_i16;4];
    let i2c_di = display_interface_i2c::I2CInterface::new(i2c, 0x3C, 0x40);
    let raw_disp = Builder::new(oled_async::displays::ssd1309::Ssd1309_128_64 {})
        .with_rotation(DisplayRotation::Rotate180)
        .connect(i2c_di);
    // let mut disp= Builder::new().connect(interface).into();
    
    let mut disp: GraphicsMode<_, _> = raw_disp.into();
    disp.init().await.unwrap();
    disp.clear();
    disp.flush().await.unwrap();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("DATA COUNT : ", Point::zero(), text_style, Baseline::Top)
        .draw(&mut disp)
        .unwrap();
    Text::with_baseline("--------STICK--------", Point::new(0, 15), text_style, Baseline::Top)
        .draw(&mut disp)
        .unwrap();
    disp.flush().await.unwrap();
    loop{
        
        let data = COUNT.load(Ordering::Acquire);
        let stick_data = [
            L_X.load(Ordering::Acquire),
            L_Y.load(Ordering::Acquire),
            R_X.load(Ordering::Acquire),
            R_Y.load(Ordering::Acquire)
        ];
        if count_flag!=data{
            count_flag =data;
            // disp.clear();
            let mut t_buf = String::<12>::new();
            let _ = write!(t_buf, "{}", count_flag);
            Rectangle::new(Point::new(80, 0), Size::new(30, 15))
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
                .draw(&mut disp)
                .unwrap();
            Text::with_baseline(&t_buf, Point::new(80, 0), text_style, Baseline::Top)
                .draw(&mut disp)
                .unwrap();
            
            disp.flush().await.unwrap();
        }
        if stick_flag!=stick_data{
            stick_flag=stick_data;
            let mut r_buf = String::<32>::new();
            let _ = write!(r_buf, "{},{},{},{}", stick_flag[0],stick_flag[1],stick_flag[2],stick_flag[3]);
            Rectangle::new(Point::new(0, 30), Size::new(130, 15))
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
                .draw(&mut disp)
                .unwrap();
            Text::with_baseline(&r_buf, Point::new(0, 30), text_style, Baseline::Top)
                .draw(&mut disp)
                .unwrap();  
        }
        
        Timer::after(Duration::from_nanos(1)).await;
    }
}