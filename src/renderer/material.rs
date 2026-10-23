
extern crate image;

use self::image::{Pixel,GenericImage};
use crate::renderer::material::image::GenericImageView;
use exr::*;
use super::*;





pub struct Texture {
    pixels: Vec<Vec<Color>>,
    width: usize,
    height: usize,
}

impl Texture 
{
    pub fn open(filename: &str) -> Texture {
        let img = image::open(filename).unwrap(); 
        Texture::from_image(img)
    }

    fn new(pixels: Vec<Vec<Color>>) -> Texture {
        let height = pixels.len();
        let width = pixels[0].len(); 
        Texture{pixels,width,height}
    }
    fn get(&self, x:usize,y:usize) -> Color {
        self.pixels[y][x]
    }

    fn from_image(img:image::DynamicImage) -> Texture {
        let (width, height) = img.dimensions();
        let mut out: Vec<Vec<Color>> = Vec::new();

        for i in 0..height {
            let mut col:Vec<Color> = Vec::new();
            for j in 0..width {
                let p = img.get_pixel(j,i);
                let c = p.channels();
                let (r,g,b) = (c[0], c[1], c[2]);
                col.push(Color::new_from_255(r,g,b));
            }
            out.push(col);
        }
        Texture::new(out)
    }



    pub fn open_exr (file: &str) -> Texture
    {
         let mut image = exr::prelude::read_first_rgba_layer_from_file(file,
                |resolution, _| {
                    let default_pixel = [0.0, 0.0, 0.0, 0.0];
                    let empty_line =  vec![ default_pixel; resolution.width() ];
                    let empty_image =  vec![ empty_line; resolution.height() ];
                    empty_image
                },
                |pixel_vector, position, (r, g, b, a): (f32, f32, f32, f32)| {
                    pixel_vector[position.y()][position.x()] = [r, g, b, a]
                },

            ).expect("could not read file");

         // collect the pixels into a texture struct with colors

         let mut out: Vec<Vec<Color>> = Vec::new();
         for line in &mut image.layer_data.channel_data.pixels
         {
            let mut out_line: Vec<Color> = Vec::new();
            for [r,g,b,_] in line {
                out_line.push(Color::new(*r,*g,*b));
             }
             out.push(out_line);
        }
         Texture::new(out)
    }

    pub fn get_pixel(&self, u:f32, v:f32)-> Color {
        let mut x = (u * self.width as f32) as usize;
        let mut y = (v * self.height as f32) as usize;
        
        
        x = std::cmp::max(x,0);
        x = std::cmp::min(x,self.width-1);
        y = std::cmp::max(y,0);
        y = std::cmp::min(y,self.height-1);
        self.pixels[y][x]
    }
}







pub struct EnvironmentSample{
    pub direction: Vector,
    pub radiance: Color,
    pub pdf: f32
}

pub struct EnvironmentCdf {
    width: usize,
    height: usize,
    cdf: Vec<f32>,
}

// cumulative distribution function
// todo implement 2 level cdf for better performance
impl EnvironmentCdf
{
    pub fn new(texture: &Texture) -> Self 
    {
        let num_pixels = texture.width * texture.height;

        let mut cdf = Vec::with_capacity(num_pixels);
        let mut total = 0.0_f64;

        for y in 0..texture.height {
            let theta = std::f64::consts::PI * (y as f64 + 0.5) / texture.height as f64;
            let sin_theta = theta.sin();
            for x in 0..texture.width {
                let color = texture.get(x,y);
                let luminance = 0.2126 * color.r as f64 + 0.7152 * color.g as f64 + 0.0722 * color.b as f64;
                let weight = luminance.max(0.0) * sin_theta;
                total += weight;
                cdf.push(total as f32);
            }
        }
        if total > 0.0 {
            for value in &mut cdf {
                *value /= total as f32;
            }
            // avoiding floating-point rounding overflow on the last
            *cdf.last_mut().unwrap() = 1.0;
        }
        else {
            let n = cdf.len();
            for (i, value) in cdf.iter_mut().enumerate() {
                *value = (i + 1) as f32 / n as f32;
            }
        }
        Self {width:texture.width, height:texture.height, cdf}
    }
    
    // returns a texel from cdf that >= the random number
    pub fn get_sample_texel<T: rand::Rng>(&self,rng: &mut T) -> (usize, usize)
    {
        let r: f32 = rng.random();
        let index = self.cdf.partition_point(|&value| value < r);
        let index = index.min(self.cdf.len() - 1);
        let x = index % self.width;
        let y = index / self.width;
        (x, y)
    }

    pub fn sample<R: Rng>(&self, texture: &Texture, rotation:f32, rng: &mut R) -> EnvironmentSample
    {
        let r: f32 = rng.gen();
        let index = self.cdf.partition_point(|&v| v < r)
            .min(self.cdf.len() - 1);

        let x = index % self.width;
        let y = index / self.width;

        // random sample within that texel.
        let u = (x as f32 + rng.gen::<f32>()) / self.width as f32;
        let v = (y as f32 + rng.gen::<f32>()) / self.height as f32;

        // convert UV to a world-space direction.
        let phi = 2.0 * PI * u;
        let theta = PI * v;

        let theta = PI * v;
        let phi = TWO_PI * (u - 0.5 - rotation);
        let direction = Vector::new(
            theta.sin() * phi.sin(),
            theta.cos(),
            theta.sin() * phi.cos(),
        );

        let radiance = texture.get(x,y);

        // convert texel probability to PDF
        let previous = if index == 0 {
    0.0
} else {
    self.cdf[index - 1]
};

let texel_probability = self.cdf[index] - previous;

let theta0 = PI * y as f32 / self.height as f32;
let theta1 = PI * (y + 1) as f32 / self.height as f32;

let solid_angle =
    (2.0 * PI / self.width as f32)
    * (theta0.cos() - theta1.cos());

let pdf = texel_probability / solid_angle;

        EnvironmentSample {direction,radiance,pdf}
    }
}















pub enum BRDF {
    Lambert, 
    OrenNayar,
    Blinn, 
    Phong,
    Constant,
    None,
}

pub struct Material {
    pub name  : String,
    pub color : Color,
    pub diffuse: BRDF,
    pub reflection_color: Color,
    pub reflection_enabled: bool,
    pub refraction_enabled: bool,
    pub illum: f32,
    pub roughness: f32,
    pub ior: f32,
    pub texmap_diffuse: Option<Texture>,
    pub texmap_normal: Option<Texture>,
}

impl Material {
    pub fn new() -> Material { Material {
        name:String::new(),
        color:Color::light_grey(), 
        diffuse:BRDF::Lambert, 
        reflection_enabled:false,
        refraction_enabled:false,
        reflection_color: Color::white(),
        roughness: 0.0, 
        ior: 1.4,
        illum: 0.0,
        texmap_diffuse: None,
        texmap_normal: None, }
    }

    pub fn set_color(mut self, color:Color) -> Self {
        self.color = color;
        self
    }
    pub fn set_reflection_enabled(mut self, enabled:bool) -> Self {
        self.reflection_enabled = enabled;
        self
    }
    pub fn set_ior(mut self, ior:f32) -> Self {
        self.ior = ior;
        self
    }
    pub fn set_roughness(mut self, roughness:f32) -> Self {
        self.roughness = roughness;
        self
    }
    pub fn set_name(mut self, name:&str) -> Self {
       self.name = name.to_string();
       self 
    }




    
    
    pub fn new_light_material() -> Material {
        let mut m = Material::new();
        m.diffuse = BRDF::Constant;
        m.reflection_enabled = false;
        m
    }
    
    fn calc_lambert(&self, pos: Vector, normal: Vector, light: &Light) -> f32 {
        let ldir = light.pos - pos;
        let n = normal.norm();
        let l = ldir.norm();
        let dot = n.dot(l);
        dot.max(0.0)
    } 
    fn calc_oren_nayar(&self, pos:Vector, normal:Vector, light:&Light, origin:Vector) -> f32 {
        let roughness = 1.0; 	//0:smooth --> 1:matte
        let albedo = 0.5;
        let ldir = (light.pos - pos).norm();
        let view_dir = (origin - pos).norm();
        let lv = ldir.dot(view_dir);
        let nl = normal.dot(ldir);
        let nv = normal.dot(view_dir);
        let s = lv * nl * nv;
        let step = if s < 0.0 {1.0} else {0.0};
        let t = step * nl.max(nv) + (1.0 - step) * 1.0;
        let sigma = roughness * roughness;
        let a = 1.0 + sigma * (albedo / (sigma + 0.13) + 0.5 / (sigma + 0.33));
        let b = 0.45 * sigma / (sigma + 0.09);
        albedo * nl.max(0.0) * (a + b * s / t) / PI
    }
    
    
    pub fn calc_diffuse(&self, pos: Vector, mut normal: Vector, tan:Vector, light: &Light, origin:Vector, uv:Vec2) -> Color {

//        if let Some(ref map) = self.texmap_normal { 	// darboux frame
//            let bitan = tan.cross(normal);
//            let space = Matrix::from_vectors(normal, tan, bitan);
//            let v = map.get_pixel(uv.x, uv.y).to_vector();
//            normal = v.transform(&space);
//        }
//        let amount = match self.diffuse {
//            BRDF::Lambert 	=> Material::calc_lambert(self,pos, normal, light),
//            BRDF::OrenNayar => Material::calc_oren_nayar(self,pos, normal, light, origin),
//            BRDF::Constant	=> 1.0,
//            _ 				=> 0.0,
//        };
        if let BRDF::None = self.diffuse {return Color::black()}
        let amount = Material::calc_lambert(self,pos, normal, light)* light.calc_falloff(pos);

//
//        let color = match self.texmap_diffuse {
//                Some( ref tex) 	=> tex.get_pixel(uv.x,uv.y),
//                _ 				=> self.color,
//        };
        let color = self.color;
        color * amount * light.color
    }

}
