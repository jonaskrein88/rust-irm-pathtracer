mod renderer;
use renderer::*;
//use renderer::color::Color;
//use std::sync::{Mutex,Arc};
//use std::{time,thread};
//use std::sync::mpsc;
//use rand::Rng;
//use chrono;



fn main() {
    let mut re = Renderer::new(500, 400);
    re.create_demo_scene();
    re.start_render();
}









/*
pub mod display
{
    extern crate image;
    extern crate piston_window;


    use piston_window::EventLoop;
    use piston::input::mouse;
    use super::*;

    const WIDTH: u32 = 2500;
    const HEIGHT: u32 = 1800;
    
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
    completion_counter : usize,
    color_buffer: image::RgbaImage,
    render_core : Renderer,

}
impl BucketRenderer 
{
    fn new(width: usize, height: usize) -> Self {
        let color_buffer = image::RgbaImage::new(width.try_into().unwrap(),height.try_into().unwrap());
        let bucket_positions = precalculate_bucket_positions(width,height); 
        let render_core = Renderer::new(width, height);
        let completion_counter = 0usize;
        BucketRenderer{width, height, bucket_positions, color_buffer, completion_counter, render_core}
    }

    // spawns the render buckets
    // retuns a receiver to fetch the buffer
    fn render_buckets(self) -> std::sync::mpsc::Receiver<image::RgbaImage>
    {
        let img =       Arc::new(Mutex::new(self.color_buffer));
        let counter =   Arc::new(Mutex::new(self.completion_counter));
        let re =        Arc::new(self.render_core);
        let (tx,rx) =   mpsc::channel();
        let num_buckets = self.bucket_positions.len();

        for pos in self.bucket_positions {
            let _re = Arc::clone(&re);
            let _img = Arc::clone(&img);
            let _counter = Arc::clone(&counter);
            let _tx = tx.clone();

            thread::spawn(move || 
            {
                let mut bucket = [[(0u8,0u8,0u8);BUCKETSIZE as usize];BUCKETSIZE as usize];
                for u in 0..BUCKETSIZE
                {
                    for v in 0..BUCKETSIZE
                    {
                        let x = pos.0 + u;
                        let y = pos.1 + v;
                        if u32::from(x) < WIDTH && u32::from(y) < HEIGHT
                        {
                            let pixel = _re.render(x.into(),y.into());
                            bucket[u as usize][v as usize] = pixel;
                        }
                    }
                }
                let mut imgRef = _img.lock().unwrap();
                let mut counter_ref = _counter.lock().unwrap();
                
                // add the pixels from the bucket to the image
                for u in 0..BUCKETSIZE
                {
                    for v in 0..BUCKETSIZE
                    {
                        let x = pos.0 + u;
                        let y = pos.1 + v;
                        if u32::from(x) < WIDTH && u32::from(y) < HEIGHT
                        {
                            let pixel = bucket[u as usize][v as usize];
                            imgRef.put_pixel(x as u32, y as u32, image::Rgba([pixel.0, pixel.1,pixel.2,255]));
                        }
                    }
                }
                // increase to completion counter
                *counter_ref += 1usize;

                // was this the last bucket?
                // if so save final buffer
                if *counter_ref == num_buckets {
                    println!("done");
                    let time = chrono::Utc::now().timestamp();
                    imgRef.save("C:/Users/j.krein/Desktop/test/rendering_".to_owned() + &time.to_string() + ".jpg");
                }

                _tx.send(imgRef.clone());
            });
            
            // end of thread
        }
        println!("Bucket threads spawned");
        rx
    }
}




    pub fn run() {
        let bucketrenderer = BucketRenderer::new(WIDTH.try_into().unwrap(),HEIGHT.try_into().unwrap());
        let img = bucketrenderer.render_buckets();
        loop {
                img.recv();
        }
    }

    pub fn run_preview() {
        let bucketrenderer = BucketRenderer::new(WIDTH.try_into().unwrap(),HEIGHT.try_into().unwrap());
        let img = bucketrenderer.render_buckets();
        println!("creating window");
        let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("Raytracer", [WIDTH, HEIGHT])
            .exit_on_esc(true)
            .build()
            .unwrap_or_else(|_e| { panic!("Could not create window!")});

        //window.set_lazy(true);
        println!("starting loop");
        while let Some(e) = window.next() {
            //println!("waiting for image");
                if let Ok(_img) = img.recv()
                {
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

}


*/
