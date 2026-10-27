
extern crate image;

use self::image::{Pixel,GenericImage};
use crate::renderer::material::image::GenericImageView;
use exr::*;
use super::*;





pub struct Texture {
    pub name: String,
    pixels: Vec<Vec<Color>>,
    width: usize,
    height: usize,
}

impl Texture 
{
    pub fn open(filename: &str) -> Texture {
        if let Ok(img) = image::open(filename) {Texture::from_image(img)}
        else {
            panic!("Could not open image {filename}");
        }
    }

    fn new(pixels: Vec<Vec<Color>>) -> Texture {
        let height = pixels.len();
        let width = pixels[0].len(); 
        let name = String::new();
        Texture{pixels,width,height,name}
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

    pub fn sample(&self, u:f32, v:f32)-> Color { 
        let wrapped_u = u - u.floor();
        let wrapped_v = v - v.floor();
        let mut x = (wrapped_u * self.width as f32) as usize;
        let mut y = (wrapped_v * self.height as f32) as usize;
        
        
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

    pub fn sample<R: Rng>(&self, texture: &Texture, rotation: f32, rng: &mut R) -> EnvironmentSample 
{
    let r: f32 = rng.gen();
    let index = self.cdf.partition_point(|&v| v < r).min(self.cdf.len() - 1);

    let x = index % self.width;
    let y = index / self.width;

    // 1. Get continuous UV coordinates within the chosen texel
    let u = (x as f32 + rng.gen::<f32>()) / self.width as f32;
    let v = (y as f32 + rng.gen::<f32>()) / self.height as f32;

    // 2. Convert UV to angles, applying the 0.0-1.0 rotation offset to Phi
    let theta = std::f32::consts::PI * v;
    // rem_euclid ensures the value stays safely between 0.0 and 1.0
    let phi = 2.0 * std::f32::consts::PI * ((u - rotation).rem_euclid(1.0));

    // 3. Convert angles to world-space direction
    let direction = Vector::new(
        -theta.sin() * phi.sin(),
        theta.cos(),
        theta.sin() * phi.cos(),
    );

    // 4. Look up radiance (Must match the un-rotated texel index we chose!)
    let radiance = texture.get(x, y);

    // 5. Calculate PDF based on the selected texel
    let previous = if index == 0 { 0.0 } else { self.cdf[index - 1] };
    let texel_probability = self.cdf[index] - previous;

    let theta0 = std::f32::consts::PI * y as f32 / self.height as f32;
    let theta1 = std::f32::consts::PI * (y + 1) as f32 / self.height as f32;

    let solid_angle = (2.0 * std::f32::consts::PI / self.width as f32) * (theta0.cos() - theta1.cos());
    let pdf = texel_probability / solid_angle;

    EnvironmentSample { direction, radiance, pdf }
}




}















pub enum ShaderType {
    Lit,
    Unlit,
    Emissive,
}

pub struct Material {
    pub name  : String,
    pub color : Color,
    pub shader_type: ShaderType,
    pub reflection_color: Color,
    pub reflection_enabled: bool,
    pub refraction_enabled: bool,
    pub illum: f32,
    pub roughness: f32,
    pub ior: f32,
    pub normal_strength: f32,
    pub texture_diffuse: Option<usize>,
    pub texture_normals: Option<usize>,
    pub texture_roughness: Option<usize>,
    pub texture_roughness_strength: f32,
    pub invert_roughness_map: bool,
}

impl Material {
    pub fn new() -> Material { Material {
        name:String::new(),
        color:Color::white(),
        shader_type: ShaderType::Lit,
        reflection_enabled:false,
        refraction_enabled:false,
        reflection_color: Color::white(),
        roughness: 0.0, 
        ior: 1.4,
        illum: 0.0,
        normal_strength: 1.0,
        texture_diffuse: None,
        texture_normals: None,
        texture_roughness: None,
        texture_roughness_strength: 1.0,
        invert_roughness_map: false,
        }
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
    pub fn set_texture_diffuse(mut self, texture:usize) -> Self {
       self.texture_diffuse = Some(texture);
       self 
    }
    pub fn set_texture_normals(mut self, texture:usize) -> Self {
       self.texture_normals = Some(texture);
       self 
    }
    pub fn set_texture_roughness(mut self, texture:usize) -> Self {
       self.texture_roughness = Some(texture);
       self 
    }




    
    fn calc_lambert(&self, pos: Vector, normal: Vector, light: &Light) -> f32 {
        let ldir = light.pos - pos;
        let n = normal.norm();
        let l = ldir.norm();
        let dot = n.dot(l);
        dot.max(0.0)
    } 
    


    pub fn get_albedo(&self, renderer: &Renderer, uv:Vec2) -> Color
    {
        match self.texture_diffuse
        {
            Some(tex_id) 	=> renderer.get_texture(tex_id).sample(uv.x,uv.y) * self.color,
            _ 				=> self.color,
        }
    }

    pub fn get_roughness(&self, renderer: &Renderer, uv:Vec2) -> f32
    {
        match self.texture_roughness
        {
            None => self.roughness,
            Some(_) if self.texture_roughness_strength == 0.0 => self.roughness,
            Some(tex_id) => {
                let mut x = renderer.get_texture(tex_id).sample(uv.x,uv.y).r.powf(0.4545);
                x = if self.invert_roughness_map {1.0 - x} else {x};
                x * self.texture_roughness_strength + (1.0 - self.texture_roughness_strength) * self.roughness
            },
        }
    }
 
    

    pub fn apply_normal_map(&self, renderer: &Renderer, tan: Vector, normal: Vector, uv: Vec2, intensity:f32) -> Vector
    {
        match self.texture_normals
        {
            None => normal,
            Some(tex_id) => {
                let texel = renderer.get_texture(tex_id).sample(uv.x, uv.y).pow(0.4545).to_vector();
                
                let mut n = Vector::new(
                    texel.x * 2.0 - 1.0,
                    texel.y * 2.0 - 1.0,
                    texel.z * 2.0 - 1.0,
                );
                //n = Vector::new(n.x, -n.y, n.z); // works for ... inverted gl??
                //n = Vector::new(n.x, n.z, n.y); 
                n = Vector::new(n.x * intensity, n.y * intensity, n.z);

                //let n = Vector::new(0.0, 0.0, 1.0); // hardcoded up vector for testing
                //
                let bitan = tan.cross(normal).norm(); 
                let true_tan = normal.cross(bitan).norm();
                // using explicit multiplication for testing
                let transformed_normal = (true_tan * n.x) + (bitan * n.y) + (normal * n.z);

                transformed_normal.norm()
            }
        }
    }

    
    // calculate diffuse for point light
    
    pub fn calc_diffuse(&self, renderer: &Renderer, hit:&Hitresult, light:&Light) -> Color
    {

//        if let Some(ref map) = self.texmap_normal { 	// darboux frame
//            let bitan = tan.cross(normal);
//            let space = Matrix::from_vectors(normal, tan, bitan);
//            let v = map.get_pixel(uv.x, uv.y).to_vector();
//            normal = v.transform(&space);
//        }
        let amount = Material::calc_lambert(self, hit.pos, hit.normal, light)* light.calc_falloff(hit.pos);
        light.color * amount
    }
    

}
