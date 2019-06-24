#[macro_use]
extern crate error_chain;
extern crate threadpool;
extern crate num;
extern crate num_cpus;
extern crate image;

use std::sync::mpsc::{channel, RecvError};
use threadpool::ThreadPool;
use num::complex::Complex;
use image::{ImageBuffer, Pixel, Rgb};
use std::u8;

error_chain! {
    foreign_links {
        MpscRecv(RecvError);
        Io(std::io::Error);
    }
}

fn wavelength_to_rgb(wavelength: u32) -> Rgb<u8> {
    let wave = wavelength as f32;
   
    // outside to inside
    //let hex =  ["#000000", "#", "#", "#", "#", "#", "#000000"];
    let hex =  ["#000000", "#05F2DB",  "#05C7F2", "#3805F2", "#7C05F2", "#F205CB", "#000000"];
    let mut colors = Vec::new();
    for element in hex.iter() {
        colors.push(hex_to_rgb(element));
    }
    while colors.len() < 7 {
        colors.insert(0, (0,0,0));
    }
    

    let (r, g, b) = match wavelength {
        380...439 => color_blend_norm(380., 439., wave, colors[0], colors[1]),
        440...489 => color_blend_norm(439., 489., wave, colors[1], colors[2]), 
        490...509 => color_blend_norm(489., 509., wave, colors[2], colors[3]), 
        510...579 => color_blend_norm(509., 579., wave, colors[3], colors[4]), 
        580...644 => color_blend_norm(579., 644., wave, colors[4], colors[5]), 
        645...780 => color_blend_norm(644., 780., wave, colors[5], colors[6]), 
        _ => (0, 0, 0),
    };

    Rgb::from_channels(r, g, b, 0)
}

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let r = match u8::from_str_radix(&hex[1..3], 16) {
        Ok(num) => num,
        Err(_) => panic!("Invalid hex"),
    };
    let g = match u8::from_str_radix(&hex[3..5], 16) {
        Ok(num) => num,
        Err(_) => panic!("Invalid hex"),
    };
    let b = match u8::from_str_radix(&hex[5..7], 16) {
        Ok(num) => num,
        Err(_) => panic!("Invalid hex"),
    };
    (r,g,b)
}

fn color_blend_norm(start: f32, end: f32, wave: f32, (r1, g1, b1): (u8, u8, u8), (r2, g2, b2): (u8, u8, u8)) -> (u8, u8, u8) {
   ((r1 as f32 + (r2 as f32 - r1 as f32) * (wave - start) / (end - start)) as u8, (g1 as f32 + (g2 as f32 - g1 as f32) * (wave - start) / (end - start)) as u8, (b1 as f32 + (b2 as f32 - b1 as f32) * (wave - start) / (end - start)) as u8) 
}

// Maps Julia set distance estimation to intensity values
fn julia(c: Complex<f32>, x: u32, y: u32, width: u32, height: u32, max_iter: u32) -> u32 {
    let width = width as f32;
    let height = height as f32;

    let mut z = Complex {
        // scale and translate the point to image coordinates
        re: 3.0 * (x as f32 - 0.5 * width) / width,
        im: 2.0 * (y as f32 - 0.5 * height) / height,
    };

    let mut i = 0;
    for t in 0..max_iter {
        if z.norm() >= 2.0 {
            break;
        }
        z = z * z + c;
        i = t;
    }
    i
}

fn run() -> Result<()> {
    let (width, height) = (3840, 2160);
    let mut img = ImageBuffer::new(width, height);
    let iterations = 100;
    
    // -0.9999 0.35
    //let c = Complex::new(-0.10576923076923084, -0.648076923076923);
    //let c = Complex::new(0.16346153846153832, 0.5865384615384616);
    let c = Complex::new(-0.6000935097734532, -0.427862402050194);

    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = channel();

    for y in 0..height {
        let tx = tx.clone();
        pool.execute(move || for x in 0..width {
                         let i = julia(c, x, y, width, height, iterations);
                         let pixel = wavelength_to_rgb(380 + i * 400 / iterations);
                         tx.send((x, y, pixel)).expect("Could not send data!");
                     });
    }

    for _ in 0..(width * height) {
        let (x, y, pixel) = rx.recv()?;
        img.put_pixel(x, y, pixel);
    }
    let _ = img.save("output.png")?;
    Ok(())
}

quick_main!(run);
