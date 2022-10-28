mod renderer;
use renderer::*;
use renderer::color::Color;
use std::sync::{Mutex,Arc};
use std::{time,thread};
use std::sync::mpsc;

use rand::Rng;



fn main() {
    display::run();
}










pub mod display
{
    extern crate image;
    extern crate piston_window;


    use piston_window::EventLoop;
    use piston::input::mouse;
    use super::*;

    const WIDTH: u32 = 500;
    const HEIGHT: u32 = 500;
    
    pub fn render_frame_lines() -> image::RgbaImage {
        let re = Renderer::new(WIDTH as usize, HEIGHT as usize);
        let counter = Arc::new(re);
        let mut img = image::RgbaImage::new(WIDTH,HEIGHT);

        let mut threads = vec!();

        for x in 0..WIDTH {
                let r = Arc::clone(&counter);
                let thread = thread::spawn(move || {
                    let mut col = vec!();
                    for y in 0..HEIGHT 
                    {
                        let pixel = r.render(x as i32,y as i32);
                        col.push(pixel);
                    }
                    col
                }); 
                threads.push(thread);
                
        }
        let mut x = 0;
        for i in threads {
            let col = i.join().unwrap();
            let mut y =0;
            for pixel in col { 
                img.put_pixel(x,y,image::Rgba([pixel.0, pixel.1,pixel.2,255]));
                y+=1;
            }
            x+=1;
        }
        print!("done");
    img
    }




    // really really slow test render every pixel in serial
    pub fn render_frame() -> image::RgbaImage {
        let re = Renderer::new(WIDTH as usize, HEIGHT as usize);
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










const BUCKETSIZE: u16 = 16;

pub fn precalculate_bucket_positions( width:usize, height:usize) -> Vec<(u16,u16)> {
    let mut positions = vec!();
    for x in (0..width).step_by(BUCKETSIZE as usize){
        for y in (0..height).step_by(BUCKETSIZE as usize){
            positions.push((x as u16, y as u16));
        }
    }
    positions
}


struct BucketRenderer
{
    width: usize,
    height: usize,
    bucket_positions : Vec<(u16,u16)>,
    color_buffer: image::RgbaImage,
    render_core : Renderer
}
impl BucketRenderer 
{
    fn new(width: usize, height: usize) -> Self {
        let color_buffer = image::RgbaImage::new(width.try_into().unwrap(),height.try_into().unwrap());
        let bucket_positions = precalculate_bucket_positions(width,height); 
        let render_core = Renderer::new(width, height);
        BucketRenderer{width, height, bucket_positions, color_buffer, render_core}
    }
    fn get_frame(){}
/*
    fn render(&mut self) {
        for _ in 0..20 {
            let x:i32 = rand::thread_rng().gen_range(0..self.width) as i32;
            let y:i32 = rand::thread_rng().gen_range(0..self.height) as i32;
            let pixel = self.render_core.render(x as i32,y as i32);
            self.color_buffer.put_pixel(x as u32,y as u32,image::Rgba([pixel.0, pixel.1,pixel.2,255]));
        }
    }
    */
    fn render_buckets(self) -> Arc<Mutex<image::RgbaImage>>{
        let img =  Arc::new(Mutex::new(self.color_buffer));
        let re = Arc::new(self.render_core);

        for pos in self.bucket_positions {
            let _re = Arc::clone(&re);
            let _img = Arc::clone(&img);

            thread::spawn(move || 
            {
                let mut bucket_buffer = [[(0u8,0u8,0u8);16];16];
                for u in 0..BUCKETSIZE
                {
                    for v in 0..BUCKETSIZE
                    {
                        let x = pos.0 + u;
                        let y = pos.1 + v;
                        if u32::from(x) < WIDTH && u32::from(y) < HEIGHT
                        {
                            let pixel = _re.render(x.into(),y.into());
                            bucket_buffer[u as usize][v as usize] = pixel;
                        }
                    }
                }
                let mut imgRef = _img.lock().unwrap();
                for u in 0..BUCKETSIZE
                {
                    for v in 0..BUCKETSIZE
                    {
                        let x = pos.0 + u;
                        let y = pos.1 + v;
                        if u32::from(x) < WIDTH && u32::from(y) < HEIGHT
                        {
                            let pixel = bucket_buffer[u as usize][v as usize];
                            imgRef.put_pixel(x as u32, y as u32, image::Rgba([pixel.0, pixel.1,pixel.2,255]));
                        }
                    }
                }
                //thread::sleep(time::Duration::from_secs(2));
            });

        }
        img
    }
}






    pub fn run() {
        //let frame_buffer = image::ImageBuffer::from_pixel(WIDTH, HEIGHT, image::Rgba([0,0,0,255]));
        //let frame_buffer = render_frame_lines();
        let bucketrenderer = BucketRenderer::new(WIDTH.try_into().unwrap(),HEIGHT.try_into().unwrap());
        let img = bucketrenderer.render_buckets();

        let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("Raytracer", [WIDTH, HEIGHT])
            .exit_on_esc(true)
            .build()
            .unwrap_or_else(|_e| { panic!("Could not create window!")});


        //window.set_lazy(true);

        while let Some(e) = window.next() {
                let _img = img.lock().unwrap(); 
                let tex = piston_window::Texture::from_image(
                    &mut window.create_texture_context(),
                    &_img,
                    &piston_window::TextureSettings::new())
                    .unwrap();

                window.draw_2d(&e, |c, g, _| {

                piston_window::clear([1.0; 4], g);
                piston_window::image(&tex, c.transform, g)
            });
        }
    }


}

