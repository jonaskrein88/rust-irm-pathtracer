
extern crate image;

use self::image::{Pixel,GenericImage};
use crate::renderer::material::image::GenericImageView;
use super::*;





pub struct Texture {
    pixels: Vec<Vec<Color>>,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn open(filename: &str) -> Texture {
        let img = image::open(filename).unwrap(); 
        Texture::from_image(img)
    }
    pub fn from_image(img:image::DynamicImage) -> Texture {
        let (width, height) = img.dimensions();
        let mut out: Vec<Vec<Color>> = Vec::new();

        for i in 0..width {
            let mut col:Vec<Color> = Vec::new();
            for j in 0..height {
                let p = img.get_pixel(i,j);
                let c = p.channels();
                let (r,g,b) = (c[0], c[1], c[2]);
                col.push(Color::new_from_255(r,g,b));
            }
            out.push(col);
        }
        Texture{pixels:out, width:width, height:height}
    }
    pub fn get_pixel(&self, u:f32, v:f32)-> Color {
        let x = ((u % 1.0) * self.width as f32) as usize;
        let y = ((v % 1.0) * self.height as f32) as usize;
        self.pixels[x][y]
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
    pub reflection: bool,
    pub refraction:bool,
    pub illum: f32,
    pub glossiness: f32,
    pub ior: f32,
    pub texmap_diffuse: Option<Texture>,
    pub texmap_normal: Option<Texture>,
}
impl Material {
    pub fn new() -> Material { Material {
        color:Color::light_grey(), 
        diffuse:BRDF::Lambert, 
        specular:BRDF::None, 
        reflection:false,
        refraction:false,
        reflection_color: Color::white(),
        glossiness: 1.0, 
        ior: 1.4,
        illum: 0.0,
        texmap_diffuse: None,
        texmap_normal: None, }
    }
    
    pub fn new_light_material() -> Material {
        let mut m = Material::new();
        m.diffuse = BRDF::Constant;
        m.specular = BRDF::None;
        m.reflection = false;
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
        let spec = normal.dot(half).max(0.0).powf(self.glossiness * 80.);
        spec
    }
    fn calc_phong(&self, pos:Vector, normal:Vector, light:&Light, origin:Vector) -> f32 {
        let sharpness = self.glossiness * 40.;
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
        if !self.reflection {return Color::black()}
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
