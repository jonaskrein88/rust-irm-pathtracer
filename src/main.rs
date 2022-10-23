mod renderer;
use renderer::*;
use renderer::color::Color;

use std::thread;

fn main() {
    display::run();
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




pub mod display
{
    extern crate image;
    extern crate piston_window;


    use piston_window::EventLoop;
    use super::*;

    const WIDTH: u32 = 500;
    const HEIGHT: u32 = 500;
    
    pub fn render_frame() -> image::RgbaImage {
        let mut re = Renderer::new(WIDTH as usize, HEIGHT as usize);
        re.init();
        let mut img = image::RgbaImage::new(WIDTH,HEIGHT);

        for x in 0..WIDTH {
            for y in 0..HEIGHT 
            {
                let pixel = re.render(x as i32,y as i32);
                img.put_pixel(x,y,image::Rgba([pixel.0, pixel.1,pixel.2,255]));
            }

        }
    img
    }

    pub fn run() {
        //let frame_buffer = image::ImageBuffer::from_pixel(WIDTH, HEIGHT, image::Rgba([0,0,0,255]));
        let frame_buffer = render_frame();


        let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("Raytracer", [WIDTH, HEIGHT])
            .exit_on_esc(true)
            .build()
            .unwrap_or_else(|_e| { panic!("Could not create window!")});

        let tex = piston_window::Texture::from_image(
            &mut window.create_texture_context(),
            &frame_buffer,
            &piston_window::TextureSettings::new())
            .unwrap();

        window.set_lazy(true);

        while let Some(e) = window.next() {
            window.draw_2d(&e, |c, g, _| {
                piston_window::clear([1.0; 4], g);
                piston_window::image(&tex, c.transform, g)
            });
        }
    }


}

