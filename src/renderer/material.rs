
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
                // instantiate your image type with the size of the image in file
                |resolution, _| {
                    let default_pixel = [0.0, 0.0, 0.0, 0.0];
                    let empty_line =  vec![ default_pixel; resolution.width() ];
                    let empty_image =  vec![ empty_line; resolution.height() ];
                    empty_image
                },
                // transfer the colors from the file to your image type,
                // requesting all values to be converted to f32 numbers (you can also directly use f16 instead)
                // and you could also use `Sample` instead of `f32` to keep the original data type from the file
                |pixel_vector, position, (r, g, b, a): (f32, f32, f32, f32)| {
                    pixel_vector[position.y()][position.x()] = [r, g, b, a]
                },

            ).expect("run the `1_write_rgba` example to generate the required file");

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








pub enum BRDF {
    Lambert, 
    OrenNayar,
    Blinn, 
    Phong,
    Constant,
    None,
}






pub struct Material {
    pub color: Color,
    pub diffuse: BRDF,
    pub specular: BRDF,
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
        color:Color::light_grey(), 
        diffuse:BRDF::Lambert, 
        specular:BRDF::None, 
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




    
    
    pub fn new_light_material() -> Material {
        let mut m = Material::new();
        m.diffuse = BRDF::Constant;
        m.specular = BRDF::None;
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
    
    fn calc_blinn(&self, pos:Vector, normal:Vector, light:&Light, origin:Vector) -> f32 {
        let light_dir = (light.pos - pos).norm();
        let view_dir = (origin - pos).norm();
        let half = (light_dir + view_dir).norm();
        let spec = normal.dot(half).max(0.0).powf((1.0/self.roughness) * 80.);
        spec
    }
    fn calc_phong(&self, pos:Vector, normal:Vector, light:&Light, origin:Vector) -> f32 {
        let sharpness = (1.0/self.roughness) * 40.;
        let light_dir = (light.pos - pos).norm();
        let view = (origin - pos).norm();
       let reflect = Vector::calc_reflect_vector(-light_dir,normal);
        let spec = view.dot(reflect).max(0.).powf(sharpness);
        spec
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


    pub fn calc_specular(&self, pos:Vector, normal:Vector, light: &Light, origin:Vector) -> Color {
        if !self.reflection_enabled {return Color::black()}
        if let BRDF::None = self.specular {return Color::black();}
        let spec = Material::calc_blinn(&self,pos, normal, light, origin) * light.calc_falloff(pos);

//        let spec = match self.specular {
//            BRDF::Blinn 	=> Material::calc_blinn(&self,pos, normal, light, origin) * light.calc_falloff(pos),
//            BRDF::Phong 	=> Material::calc_phong(&self,pos, normal, light, origin) * light.calc_falloff(pos),
//            _ 				=> 0.0,
//        };
        light.color * spec
    }
}
