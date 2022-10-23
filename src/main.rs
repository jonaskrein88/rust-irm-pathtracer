mod renderer;
use renderer::*;
use renderer::color::Color;

use std::thread;

fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
pub mod unit_test_threads 
{
    use super::*;

    fn test_function(x:i32) -> Color {Color::black()}

    fn test_threads() -> Result<(),String> {
        let mut buffer = Framebuffer::new(20,20);
        let mut threads = vec!();
        for x in 0..(20*20) {
            threads.push(thread::spawn( move || test_function(x)));
        }
        for t in threads {
            let result = t.join().unwrap();
            buffer.pixels[0][0] = result;
        }
        Ok(())
    }
}

