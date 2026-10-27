use super::*;
use color::*;
use exr::prelude::write_rgba_file;

pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>,
    pub num_samples : Vec<u32>
}


impl FrameBuffer 
{
	// fill with black and sample counts to 0
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels:      vec![Color::black(); (width * height) as usize],
            num_samples: vec![0; (width * height) as usize],
        }
    }

    fn index_to_xy(&self, i: u32) -> (u32, u32) {
	    let x = i % self.width;
	    let y = i / self.width;
	    (x,y)
	}


    fn xy_to_index(&self,x:u32, y:u32) -> usize {
    	(y * self.width + x) as usize
    }

    // sets the color
    // does no weighted accumulation
    // for when initializing a temp buffer
    pub fn set(&mut self, x:u32, y:u32, color:Color)
    {
    	let i = self.xy_to_index(x,y);
    	self.pixels[i] = color;
    	self.num_samples[i] = 1;
    }

    pub fn add_sample(&mut self, x:u32, y:u32, color:Color)
    {
    	let i = self.xy_to_index(x,y);
    	if i >= self.num_samples.len() {
    		panic!("sample out of bounds: x {0}, y {1}",x,y);
    	}
    	let n = self.num_samples[i];
    	self.pixels[i] = (self.pixels[i] * n as f32 + color) / (n+1) as f32;
    	self.num_samples[i] += 1;
    }

    pub fn add_buffer(&mut self, other: &FrameBuffer)
    {
	    for (i, other_pixel) in other.pixels.iter().enumerate()
	    {
	        let n = self.num_samples[i];
	        self.pixels[i] = (self.pixels[i] * n as f32 + *other_pixel) / (n + 1) as f32;
	        self.num_samples[i] += 1;
	    }
	}
    pub fn get(&self, x:u32, y:u32) -> Color
    {
        let index = self.xy_to_index(x,y);
        self.pixels[index]
    }

    pub fn to_image(&self) -> image::RgbaImage 
    {
    	let mut img = image::RgbaImage::new(self.width, self.height);
        for x in 0..self.width
        {
            for y in 0..self.height
            {
            	let index = self.xy_to_index(x,y);
                let (r,g,b) = self.pixels[index].to_0_255();
                img.put_pixel(x,y,image::Rgba([r,g,b,255]));
            }
        }
        img
    }
    pub fn write_exr(&self, file:&str)
    {
	    write_rgba_file(file,self.width as usize,self.height as usize, |x, y| 
        {
				let c = self.get(x as u32, y as u32);
				(c.r, c.g, c.b, 1.0)
			},
		)
		.unwrap();
	}






}