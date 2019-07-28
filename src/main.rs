// the eink library
extern crate epd_waveshare;
use epd_waveshare::{
    epd1in54::{Buffer1in54, EPD1in54},
    graphics::{Display, DisplayRotation},
    prelude::*,
};
use std::thread::sleep;
use std::time::Duration;

// Graphics
extern crate embedded_graphics;
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::image::Image1BPP;
use embedded_graphics::prelude::*;
//use embedded_graphics::primitives::{Circle, Line};
use embedded_graphics::Drawing;

use core::option::*;
use linux_embedded_hal::Delay;
use linux_embedded_hal::Pin;
use linux_embedded_hal::Spidev;
use embedded_hal::digital::OutputPin;

use sysfs_gpio::Direction;

use linux_embedded_hal::spidev::{SpidevOptions, SpidevTransfer, SPI_MODE_0};
// use std::io::Error;
use std::io::prelude::*;
// use linux_embedded_hal::spidev::*;

pub enum Error {
    UnexpectedResponse,
}

fn main() {

    match Display() {
        Ok(_) =>  {println!("Operation ok")},
        Err(_) => {println!("Something failed");}
    }
}

fn  Display() -> Result<(),Error> {
    // let mut delay = Delay::new(syst,clocks);

    let mut delay = linux_embedded_hal::Delay;

    let mut spi = Spidev::open("/dev/spidev0.0").unwrap();
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(1_000_000)
        .mode(SPI_MODE_0)
        .build();
    match spi.configure(&options) {
        Ok(_) =>  {println!("Configure ok")},
        Err(x) => {println!("Configure failed {}",x);}
    }

    // Pin Mappings for NEONano
    // Pin     Connecton   Colour       LXnum
    // P0.27   busy        purple       2
    // P0.26   Rst         white        1
    // P0.02   DC          Green        0
    // GND                 black
    // P0.25   CS          orange       67
    // P0.24   clk         yellow
    // P0.23   Din (MOSI)  blue
 

    // Rpi bindings from https://www.waveshare.com/w/upload/a/a2/1.8inch_LCD_Module_User_Manual_EN.pdf

    //  1.8inchLCD module   Raspberry Pi        BCD number
    //      3.3V            3.3V
    //      GND             GND
    //      DIN             MOSI (PIN 19)
    //      CLK             SCLK (PIN23)
    //      CS              CE0 (PIN 24)        8
    //      DC              GPIO.6 (PIN 22)     25
    //      RST             GPIO.2(PIN13)       27
    //      BL              GPIO.5 (PIN18)      24



    struct PinMappings{
        cs:u64,
        rst:u64,
        busy:u64,
        dc:u64,
    }

    const  NEOMAPPING: PinMappings =  PinMappings{cs:67, rst:1, busy:2, dc:0 };
    const  RPI3MAPPING: PinMappings = PinMappings{cs:8, rst:27, busy:24, dc:25 }; 

    let mapping=RPI3MAPPING;


    println!("Export Pins");

    let cs = Pin::new(mapping.cs);
    cs.export();
    cs.set_direction(Direction::Low);
    let mut rst = Pin::new(mapping.rst);
    match rst.export() {
        Ok(_) =>  {println!("Rst ok")},
        Err(x) => {println!("Rst Export Failed {}",x);}
    }
    rst.set_direction(Direction::Low);
    rst.set_low();
    rst.set_high();
    let mut busy = Pin::new(mapping.busy);
    
    match busy.export() {
        Ok(_) =>  {println!("Busy ok")},
        Err(x) => {println!("Busy Export Failed {}",x);}
    }
    busy.set_direction(Direction::Low);
    let mut dc = Pin::new(mapping.dc);

    dc.export();
    dc.set_direction(Direction::Low);

    match EPD1in54::new(&mut spi, cs, busy, dc, rst, &mut delay) {
        Ok(x) => {
            println!("PD1in54::new: OK");
            let mut epd = x;
            // Setup the graphics
            let mut buffer = Buffer1in54::default();
            let mut display = Display::new(epd.width(), epd.height(), &mut buffer.buffer);

            // Draw some text
            display.draw(
                Font6x8::render_str("Hello Rust vesropm!")
                    .with_stroke(Some(Color::Black))
                    .with_fill(Some(Color::White))
                    .translate(Coord::new(5, 5))
                    .into_iter(),
            );
            sleep(Duration::from_millis(1_000));
            let rust_bytes = include_bytes!("../data/rust144x144.raw");
            let abema_bytes = include_bytes!("../data/abema151x151.raw");

            let rust_img: Image1BPP<epd_waveshare::color::Color> =
                embedded_graphics::image::Image::new(rust_bytes, 144, 144);
            let abema_img: Image1BPP<epd_waveshare::color::Color> =             embedded_graphics::image::Image::new(abema_bytes, 151, 151);
            display.clear_buffer(Color::White);
            display.draw(rust_img.translate(Coord::new(28,28)).into_iter());
            // Transfer the frame data to the epd
            let _ans = epd.update_frame(&mut spi, &display.buffer());

            // Display the frame on the epd
            let _ans2 = epd.display_frame(&mut spi);
        }
        Err(_) => {
            println!("Good bye");
        }
    };
    Ok(())

    /*
        let mut x=0;
        let mut y=0;
                    display.draw(
                    Font6x8::render_str("Hello, World!")
                        .with_stroke(Some(Color::Black))
                        .with_fill(Some(Color::White))
                        .translate(Coord::new(x, y))
                        .into_iter()
                );


                // Transfer the frame data to the epd
                let _ans = epd.update_frame(&mut spi, &display.buffer());

                // Display the frame on the epd
                let _ans2 = epd.display_frame(&mut spi);
                x += 0;
                y += 9;
            }else
            {
                led0.set_high();
            }
            if btn2.is_low(){
                led1.set_low();
                display.clear_buffer(Color::White);
                display.draw(rust_img.translate(Coord::new(28,28)).into_iter());
                            // Transfer the frame data to the epd
                let _ans = epd.update_frame(&mut spi, &display.buffer());

                // Display the frame on the epd
                let _ans2 = epd.display_frame(&mut spi);
            }else
            {
                led1.set_high();
            }
            if btn3.is_low(){
                led2.set_low();
                display.clear_buffer(Color::Black);
                display.draw(abema_img.translate(Coord::new(24,24)).into_iter());
                            // Transfer the frame data to the epd
                let _ans = epd.update_frame(&mut spi, &display.buffer());

                // Display the frame on the epd
                let _ans2 = epd.display_frame(&mut spi);
            }else
            {
                led2.set_high();
            }
        }
    */
}
