// the eink library
extern crate clap;
extern crate epd_waveshare;
extern crate input_stream;

use epd_waveshare::{
    epd1in54::{Buffer1in54, EPD1in54},
    graphics::Display,
    prelude::*,
};

use std::io::BufRead;

// Graphics
extern crate embedded_graphics;
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use embedded_graphics::Drawing;

use embedded_hal::digital::OutputPin;
use input_stream::InputStream;
use linux_embedded_hal::Pin;
use linux_embedded_hal::Spidev;

use std::error::Error;
use sysfs_gpio::Direction;

use linux_embedded_hal::spidev::{SpidevOptions, SPI_MODE_0};

pub enum NeoError {
    UnexpectedResponse,
    IoError(std::io::Error),
    Suberror(String),
}

impl std::fmt::Display for NeoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NeoError::IoError(e) => write!(f, "{}", e.to_string()),
            NeoError::Suberror(s) => write!(f, "{}", s),
            _ => write!(f, "UnexpectedResponse"),
        }
    }
}

impl<E> From<E> for NeoError
where
    E: Error,
{
    fn from(e: E) -> Self {
        NeoError::Suberror(e.to_string())
    }
}

pub enum PayloadData<'a> {
    Text(InputStream<std::io::StdinLock<'a>>), // TODO this type should be something more generic when I understund it more
    Image(
        i32,
        i32,
        embedded_graphics::image::Image1BPP<'a, epd_waveshare::color::Color>,
    ),
    Internal,
}

fn main() -> std::io::Result<()> {
    let matches = clap::App::new("neo_eink")
        .version("0.1")
        .about("Display items on waveshare connected on SPI")
        .author("Fredrik SIMONSSON")
        .arg(
            clap::Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            clap::Arg::with_name("hal-mode")
                .long("hal-mode")
                .takes_value(true)
                .help("choose hal mode (RPI or NEO)"),
        )
        .arg(
            clap::Arg::with_name("image")
                .long("image")
                .takes_value(true)
                .help("image name, rust or ameba"),
        )
        .get_matches();
    let stdinlock = std::io::stdin();
    let s: InputStream<std::io::StdinLock> = InputStream::new(stdinlock.lock());

    match matches.value_of("hal-mode") {
        Some("neo") => println!("Neo mode not implemented yet"),
        _ => println!("default mode Rpi"),
    };

    let rust_bytes = include_bytes!("../data/rust144x144.raw");
    let abema_bytes = include_bytes!("../data/abema151x151.raw");
    let rust_img: embedded_graphics::image::Image1BPP<epd_waveshare::color::Color> =
        embedded_graphics::image::Image::new(rust_bytes, 144, 144);
    let abema_img: embedded_graphics::image::Image1BPP<epd_waveshare::color::Color> =
        embedded_graphics::image::Image::new(abema_bytes, 151, 151);

    let mypayload = match matches.value_of("image") {
        Some("rust") => PayloadData::Image(28, 28, rust_img),
        Some("abema") => PayloadData::Image(24, 24, abema_img),
        Some(_) => PayloadData::Internal,
        None => PayloadData::Text(s),
    };

    match display_payload(mypayload) {
        Ok(_) => println!("Operation ok"),
        Err(e) => {
            println!("Something failed {}", e);
        }
    }
    Ok(())
}

fn display_payload(payload: PayloadData) -> Result<(), NeoError> {
    // let mut delay = Delay::new(syst,clocks);

    let mut delay = linux_embedded_hal::Delay;

    let mut spi = Spidev::open("/dev/spidev0.0")?;
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(1_000_000)
        .mode(SPI_MODE_0)
        .build();
    spi.configure(&options)?;

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

    struct PinMappings {
        cs: u64,
        rst: u64,
        busy: u64,
        dc: u64,
    }

    // expected
    const NEOMAPPING: PinMappings = PinMappings {
        cs: 67,
        rst: 1,
        busy: 2,
        dc: 0,
    };
    // const RPI3MAPPING: PinMappings = PinMappings {
    //     cs: 8,
    //     rst: 27,
    //     busy: 24,
    //     dc: 25,
    // };

    let mapping = NEOMAPPING;

    let cs = Pin::new(mapping.cs);
    cs.export()?;
    cs.set_direction(Direction::Low)?;
    let mut rst = Pin::new(mapping.rst);
    rst.export()?;
    rst.set_direction(Direction::Low)?;
    rst.set_low();
    rst.set_high();
    let busy = Pin::new(mapping.busy);
    busy.export()?;
    busy.set_direction(Direction::Low)?;
    let dc = Pin::new(mapping.dc);
    dc.export()?;
    dc.set_direction(Direction::Low)?;

    let mut epd = EPD1in54::new(&mut spi, cs, busy, dc, rst, &mut delay)?;
    println!("PD1in54::new: OK");
    // Setup the graphics
    let mut buffer = Buffer1in54::default();
    let mut display = Display::new(epd.width(), epd.height(), &mut buffer.buffer);

    display.clear_buffer(Color::White);
    // Draw some text
    match payload {
        PayloadData::Text(stream) => stream.lines().enumerate().for_each(|(pos, message)| {
            let pos = pos as i32;
            if let Ok(message) = message {
                display.draw(
                    Font6x8::render_str(&message)
                        .with_stroke(Some(Color::Black))
                        .with_fill(Some(Color::White))
                        .translate(Coord::new(5, 5 + pos * 9))
                        .into_iter(),
                );
            }
        }),
        PayloadData::Image(x, y, img) => {
            display.draw(img.translate(Coord::new(x, y)).into_iter());
        }
        _ => {}
    }
    epd.update_frame(&mut spi, &display.buffer())?;
    epd.display_frame(&mut spi)?;

    Ok(())
}
