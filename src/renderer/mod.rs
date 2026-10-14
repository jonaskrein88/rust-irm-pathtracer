#![allow(warnings)] 


pub mod vectormath;
pub mod camera;
pub mod geometry;
pub mod material;
pub mod color;
pub mod framebuffer;

extern crate kdtree;
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;

use std::sync::mpsc;
use std::{time,thread,mem};
use std::path::Path;
use std::sync::Arc;

use rayon::prelude::*;

use vectormath::*;
use color::*;
use camera::*;
use geometry::*;
use material::*;
use framebuffer::FrameBuffer;

const PI: f32 		= 3.14159265359; 
const TWO_PI: f32 	= 6.2831832; 


pub enum GiMethod {
    BruteForce,
    IrradianceMap
}


pub enum RenderMode {
    Beauty,
    Debug_IrradianceMap,
    Debug_Normals,
    Debug_UVs,
}




pub struct Settings {
    environment:Color,
    environment_map: Option<Texture>,
    shadows: bool,
    render_alpha: bool,
    render_normals: bool,
    render_uv: bool,
    gi_method_primary: GiMethod,
    gi_method_secondary: GiMethod,
    gi_on: bool,
    reflections_on: bool,
}

impl Settings {
    fn new() -> Settings {
        Settings { 
            shadows:true, 
            environment: Color::black(),
            environment_map: None,
            render_alpha: false,
            render_normals: false,
            render_uv:false,
            gi_method_primary:   GiMethod::BruteForce,
            gi_method_secondary: GiMethod::IrradianceMap,
            gi_on: true,
            reflections_on: true,
        }
    }
}








pub struct IRClass {
    samples: KdTree<f64, (Color,Vector), [f64;3]>,
}
impl IRClass 
{
    fn new() -> IRClass {
        IRClass{samples:KdTree::new(3)}
    }
    fn near_points(&self, p:Vector, num:usize) -> Vec<(f64, &(Color,Vector))> {
        self.samples.nearest(&p.to_array(), num, &squared_euclidean).unwrap()
    }
    fn add(&mut self, p:Vector, c:Color, n:Vector) {
        self.samples.add(p.to_array(), (c,n)).unwrap();
    } 
    fn check_sample(&self, hit:&Hitresult)  -> Option<Color> 
    {
        let near = self.near_points(hit.pos,1);
        match near.get(0) {
            None => None,
            Some(&(d,&(c,n))) => {
                if d > 0.2 {return None;}
                if n.angle_between(hit.normal) > 0.2 {return None;}
                Some(c)
            }
        }
    }
}








pub struct Hitresult<'a> {
    pos: Vector,
    normal: Vector,
    id: u32,
    node: &'a Geo,
    uv: Vec2,
    tangent:Vector,
}
impl<'a> Hitresult<'a> {
    pub fn new(p: Vector, n: Vector, obj: u32, node: &Geo) -> Hitresult {
        Hitresult{pos: p, normal: n, id: obj, node:node, uv:Vec2::new(0.,0.), tangent:Vector::empty()}
    }
    pub fn new_with_uvs(p: Vector, n: Vector, uv:Vec2, obj: u32, node: &Geo) -> Hitresult {
        Hitresult{pos: p, normal: n, id: obj, node:node, uv:uv, tangent:Vector::empty()}
    }
    pub fn new_with_tan(p: Vector, n: Vector, uv:Vec2, tan: Vector, obj: u32, node: &Geo) -> Hitresult {
        Hitresult{pos: p, normal: n, id: obj, node:node, uv:uv, tangent:tan}
    }
}













pub struct Renderer {
	pub camera: Camera,
	nodes: Vec<Geo>,
	lights: Vec<Light>,
	image_size: (u32,u32),
	settings:Settings,
	irradiance_map: IRClass,
}



impl Renderer 
{

	pub fn new(x:u32, y:u32) -> Renderer 
	{ 
		let nodes: Vec<Geo> = Vec::new(); 
		let lights: Vec<Light> = Vec::new(); 
		let mut cam = Camera::new(x,y);

		let mut r = Renderer {
			camera: cam, 
			nodes: nodes, 
			lights:lights, 
			image_size:(x,y),
			settings: Settings::new(),
			irradiance_map: IRClass::new(),
		};
		//r.init();
		r
	}


	// ambigious TODO
	pub fn add(&mut self, mut g:Geo) { 
		g.prepass();
		self.nodes.push(g);
	}

	pub fn create_demo_scene(&mut self) {

        self.camera.pos.z = -40.0;
        self.camera.pos.y = 20.0;

        //let hdri = material::Texture::open_exr("C:/Users/j.krein/Desktop/tomoco_studio_small.exr");
        //let hdri = material::Texture::open("C:/Users/j.krein/Desktop/tomoco_studio_small.jpg");
        //let hdri = material::Texture::open("C:/Users/j.krein/Desktop/environment_test.jpg");
        //self.settings.environment_map = Some(hdri);

		let mut s2 = Geo::new()
			.translate(Vector::new(0.0, 10.0, 5.0));
		//s2.pos.z = 0.0;
		//s2.pos.x = 10.;
		//s2.pos.y = 5.;
		s2.shape = Shape::Sphere(12.);
		
		//s2.material.refraction = true;
		s2.material.reflection = true;
        s2.material.color = Color::red();
		s2.material.specular = BRDF::Blinn;
		//s2.material.diffuse = BRDF::None;
        //s2.material.ior = 0.1;
		self.add(s2);


		// This will ALWAYS look in your project's root folder, no matter where it's executed from
		let mut project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
		let stage_path = project_root.join("models/stage.obj");

		let mut stage_mesh = open(stage_path.to_str().unwrap()).unwrap();
		let mut stage_geo = Geo::create_mesh(stage_mesh);
        stage_geo.material.color = Color::white();
		stage_geo.material.specular = BRDF::None;
        self.add(stage_geo);

		// pot
		let pot_path = project_root.join("models/teapot.obj");
		let mut mesh = open(pot_path.to_str().unwrap()).unwrap();
		//mesh.scale(0.5);
		let mut mg = Geo::create_mesh(mesh);
        mg.material.color = Color::white();
        mg.material.specular = BRDF::Blinn;
		//mg.material.texmap_diffuse = Some(tex);
		//mg.material.texmap_normal = Some(nmap);
		//mg.material.reflection = true;
		self.add(mg);


		let mut l = Light::new();
		l.pos.y = 10.;
		l.pos.x = -10.;
		l.pos.z = -35.;
		l.color = Color::new(0.85,0.85,1.0);
        l.color =  l.color * 3.0;
		self.lights.push(l);

		let mut l2 = Light::new();
		l2.pos.y = 5.;
		l2.pos.x = 15.;
		l2.pos.z = -45.;
		l2.color = Color::new(1.,1.,1.);
        l2.color =  l2.color * 3.0;
		self.lights.push(l2);
	}



	pub fn initialize(&mut self)
	{
        println!("starting IR generation");
		self.create_ir();
        println!("Init complete");
    }









	pub fn reads_gi(&self, m:&Material) -> bool {
		if m.color == Color::black() {return false}
		match m.diffuse {
			BRDF::None =>  false,
			BRDF::Constant =>false,
			_ => true,
		}
	}
	pub fn create_ir(&mut self) {
		let samples = 1500;
		let mut i = 0;
		let mut ir = IRClass::new();
		for _ in 0..samples {
			if let Some(hit) = self.cast_random_ray() {
				match ir.check_sample(&hit) {
					Some(_) => (),
					None =>  {
						let mut out = self.approximate_direct_diffuse_light(&hit, self.camera.pos);
						if self.reads_gi(&hit.node.material) {out += self.ir_secondaray(&hit, &mut ir, 2);}
						ir.add(hit.pos, out, hit.normal);
					}
				}
			}
		}
		self.irradiance_map = ir;
	}
	pub fn ir_secondaray(&self, hit:&Hitresult, ir:&mut IRClass, level:usize) -> Color {
		let mut out = Color::black();
		let samples = 10;
		for _i in 0..samples {
			let dir = hit.normal.random_hemispherical();
			let ray = Ray::biased_from_direction(hit.pos, dir);
			if let Some(new_hit) = self.cast_ray(&ray) {
				let c = match ir.check_sample(&new_hit) {
					Some(col) => col,
					None =>  {
						let mut surface = self.approximate_direct_diffuse_light(&new_hit, self.camera.pos);
						if level > 0 && self.reads_gi(&new_hit.node.material) {
							let gi = self.ir_secondaray(&new_hit, ir, level-1) * hit.node.material.color;
							surface = surface + gi;
						}
						if level > 0 { ir.add(new_hit.pos, surface, new_hit.normal);}
						surface
					}
				};
				out = out + c;
			}
		}
		(out / samples as f32) * hit.node.material.color
	}
	pub fn ir_read(&self, pos:Vector) -> Color { 
		let near_points = self.irradiance_map.near_points(pos, 20);
		if near_points.len() == 0 {panic!("empty ir map");}
		//return (near_points[0].1).0;
		let mut out = Color::black();
		let mut accum = 0.0;

		for s in near_points {
			let (_, &(col,_)) = s;
			//let denom = 1.0/dist.sqrt().max(0.0001) as f32;
			//out = out + col * denom;
			//accum += denom;
			out = out + col;
			accum += 1.;
		}
		out / accum as f32
	}

    //  debug render pass
	pub fn ir_render_samples(&self, hit:&Hitresult) -> Color {
		let v = self.irradiance_map.near_points(hit.pos, 1);
		if let Some(&(p,(color,_))) = v.get(0) {
			if p < 0.01 { return *color;}
		}
		Color::black()
	}
	pub fn add_mesh(&mut self, mesh:Polymesh) {
		self.add(Geo::create_mesh(mesh))
	}

	pub fn cast_random_ray(&self) -> Option<Hitresult> {
		let x = rand::random::<f32>() * self.image_size.0 as f32;
		let y = rand::random::<f32>() * self.image_size.1 as f32;
		let ray = self.ray_from_pixel(x,y);
		self.cast_ray(&ray)
	}

	pub fn cast_random_hemispherical_ray(&self, p:Vector, n:Vector) -> Option<Hitresult> {
		let hemis = n.random_hemispherical();
		let ray = Ray::biased_from_direction(p, hemis);
		self.cast_ray(&ray)
	}


	pub fn calc_refraction(&self, hit:&Hitresult, origin:Vector) -> Color {
		fn calc_refraction_secondary(r:&Renderer, hit:&Hitresult, origin:Vector, level:usize) -> Color {
			let i = (hit.pos - origin).norm();
			let dir = Vector::calc_refract_vector(i, hit.normal, 1.3);
			let ray = Ray::biased_from_direction(hit.pos, dir);
			if let Some(new_hit) = r.cast_ray(&ray) {
				if level > 2 {return r.approximate_color(&new_hit, hit.pos,false);}
				else if new_hit.node.material.refraction {
					return calc_refraction_secondary(r, &new_hit, hit.pos, level+1)
				}
				else {return r.approximate_color(&new_hit, hit.pos,false) + r.calc_gi(&new_hit);}
			}
			r.settings.environment
		}
		let i = (hit.pos - origin).norm();
		let dir = Vector::calc_refract_vector(i, hit.normal, 1.3);
		let ray = Ray::biased_from_direction(hit.pos, dir);
		if let Some(new_hit) = self.cast_ray(&ray) {
			return calc_refraction_secondary(self,&new_hit, hit.pos,0);
		}
		return Color::black()
	}


	pub fn ray_from_pixel(&self, x: f32, y: f32) -> Ray{
		let dir = self.camera.calc_ray_direction(x,y);
		Ray::from_direction(self.camera.pos, dir)
	}

   

    // render pass for debugging???
	pub fn measure_variance(&self, x: f32, y:f32) -> Color {
		let max = 8;
		let mut accum = Color::black();
		let mut samples:Vec<Color> = Vec::new();
		for _ in 0..max {
			let xr = rand::random::<f32>();
			let yr = rand::random::<f32>();
			let ray = self.ray_from_pixel(x + xr, y + yr);
			if let Some(hit) = self.cast_ray(&ray) {
				let color = self.calculate_brdf(&hit, self.camera.pos);
				accum = accum + color;
				samples.push(color);
			}
		} 
		let mean = accum / max as f32;
		let v = Renderer::sample_variance(&samples, mean);
		Color::new_mono(v.powf(0.5))
		//Color::red().interpolate(Color::blue(), v.powf(0.5))
	}
	pub fn cast_ray(&self, ray:&Ray) -> Option<Hitresult> {
		let mut best_depth = f32::MAX;
		let mut best_hit: Option<Hitresult> = None;
		for n in &self.nodes  {
			let hit_opt = n.calculate(&ray);
			if let Some(hit) = hit_opt {
				let depth = (ray.origin - hit.pos).length();
				if depth < best_depth {
					best_depth = depth;
					best_hit = Some(hit);
				}
			}
		}
		best_hit
	}

	pub fn calculate_brdf(&self, hit:&Hitresult, origin:Vector) -> Color {
		//if let BRDF::Constant = hit.node.material.diffuse {return hit.node.material.color}
		let mut out = Color::black();
			for light in &self.lights
			{
				if self.settings.shadows && self.check_shadow(hit.pos, &light) {continue;}
				let diffuse = hit.node.material.calc_diffuse(hit.pos, hit.normal, hit.tangent, &light, origin, hit.uv);
				let spec =    hit.node.material.calc_specular(hit.pos, hit.normal, &light, origin);
				out = out + diffuse + spec;
			}
		return out;
	}

	pub fn approximate_direct_diffuse_light(&self, hit:&Hitresult, origin:Vector) -> Color {
                let mut diffuse = Color::black();
                for light in &self.lights
                {
                    if self.settings.shadows && self.check_shadow(hit.pos, &light) {continue;}
                    diffuse = diffuse + hit.node.material.calc_diffuse(hit.pos, hit.normal, hit.tangent, &light, origin, hit.uv);
                }
                let env = self.calculate_environment_lighting(&hit,24);
                diffuse + env
    }


	pub fn approximate_color(&self, hit:&Hitresult, origin:Vector, reflection: bool) -> Color 
    {
		if reflection && hit.node.material.reflection 
        {
			let fresnel = self.calc_fresnel(hit.pos, self.camera.pos,hit.normal, hit.node.material.ior);
            let ray = Self::create_reflection_ray(&origin, &hit);
            let hit_opt = self.cast_ray(&ray);

			let reflect = match hit_opt {
				Some(new_hit) => {self.approximate_color(&new_hit, hit.pos, false)},
				None 		=> {self.settings.environment}
			};
			let surface = self.calculate_brdf(&hit,self.camera.pos) * (1. - fresnel);
			return reflect + surface
		}
		self.calculate_brdf(hit, origin)
	}




/*
	pub fn render(&self, x: i32, y: i32) -> (u8,u8,u8) {
		//let r = self.ray_from_pixel(x as f32,y as f32);
		//self.cast_primary_ray(&r).to_0_255()
        self.super_sample_fixed_rate(x as f32, y as f32).to_0_255()
        //self.super_sample_bokeh(x as f32,y as f32).to_0_255()
	}
*/


	pub fn super_sample_fixed_rate(&self, x: u32, y:u32, samples:u32) -> Color {
		let mut out = Color::black();
		for _ in 0..samples {
			let mut ray = self.ray_from_pixel(x as f32 + rand::random::<f32>(),y as f32 + rand::random::<f32>());
			let c = self.cast_primary_ray(&ray);
			out += c;
		}
		out / samples as f32
	}


	pub fn super_sample_bokeh(&self, x: u32, y:u32, sample_count:u32) -> Color {
		let focal_dist = 40.0;
		let mut out = Color::black();
		let r = 0.2;

		let up = self.camera.up;
		let right = self.camera.dir.cross(up);

		let old_dir = {
			self.camera.calc_ray_direction(x as f32, y as f32)
		};
		let focal_point = self.camera.pos + old_dir * focal_dist;


		for i in 0..sample_count {
			let u = (i as f32 * TWO_PI / sample_count as f32 ).cos() * r;
			let v = (i as f32 * TWO_PI / sample_count as f32 ).sin() * r;

			let origin = self.camera.pos + right * u + up * v;

			let dir = (focal_point - origin).norm();
			let ray = Ray::from_direction(origin, dir);

			out += self.cast_primary_ray(&ray);
		}
		out / sample_count as f32
	}









	pub fn cast_primary_ray(&self, ray:&Ray) -> Color {
		let hit_result = self.cast_ray(ray);
		match hit_result {
			None if self.settings.render_alpha	=> Color::black(),
			None 								=> self.settings.environment,
			Some(hit) 							=> {
							if self.settings.render_alpha {return Color::white();} 		// only alpha
							if self.settings.render_normals {return hit.normal.normal_to_color();}
							if self.settings.render_uv {return hit.uv.to_color();}
                            //let view = hit.pos - self.camera.pos;
                            //let reflect = Vector::calc_reflect_vector(view, hit.normal);
                            //return reflect.to_color();
							//return self.ir_render_samples(&hit);      // samples debug 
                            //return self.ir_read(hit.pos);
							//return hit.normal.random_hemispherical().normal_to_color();
                            //return hit.normal.normal_to_color();
                            //return self.calculate_environment_lighting(&hit);
                            //return self.calc_gi(&hit) * hit.node.material.color;                // show gi only
							let mut surface = self.calculate_brdf(&hit,self.camera.pos);
							//if hit.node.material.refraction {surface = surface + self.calc_refraction(&hit, self.camera.pos)}
                            let env = self.calculate_environment_lighting(&hit,24);
                            //return env;

										
							let mut out = if hit.node.material.reflection {
								let fresnel = self.calc_fresnel(hit.pos, self.camera.pos,hit.normal, hit.node.material.ior);
								let reflect = self.calc_reflection(&hit,self.camera.pos,0) * fresnel;
								reflect + surface * (1. - fresnel)
							} else{
								surface
							}; 
					        out = out + self.calc_gi(&hit) * hit.node.material.color; 
                            out += env;
							out

						}
		}
	}


	pub fn calc_fresnel(&self, pos:Vector, origin:Vector, n:Vector, power:f32) -> f32 {
		let scale = 1.0;
		let bias = 0.0;
		let i = (pos - origin).norm();
		let i_dot_n = bias + scale * (i.dot(n) +1.).powf(power);
		let mi = i_dot_n.min(1.);
		mi.max(0.)
	}

	pub fn check_shadow(&self, pos:Vector, light:&Light) -> bool {
		let ray = Ray::from_target(light.pos, pos);
		let hit = self.cast_ray(&ray);
		if let Some(Hitresult{pos:hitpos,..}) = hit {
			//let dist = hitpos.dist(pos);
			let light_pos = light.pos.dist(pos);
			let light_hit = light.pos.dist(hitpos);
			if (light_hit + 0.1)< light_pos {return true}
		}
		false
	}






    fn sample_environment(&self, r: Vector) -> Color {

        return Color::new(0.2,0.3,0.6).interpolate(Color::new(0.4,0.3,0.2), r.y);

        if let Some(hdri) = &self.settings.environment_map 
        {
            let mut xz = r.clone();
            xz.y = 0.0;
            xz = xz.norm();
            let u = (xz.x/xz.z).atan() / TWO_PI + 0.5;
            let v = r.y / 2.5 + 0.5;

            return hdri.get_pixel(u,1.0 - v);
        }
        else {self.settings.environment}
    }



    pub fn sample_environment_lighting(&self, hit:&Hitresult) -> (Color,f32)
    {
        let direction = hit.normal.random_hemispherical();
        //let direction = hit.normal;

        let ray = Ray::biased_from_direction(hit.pos, direction);
        let dot = direction.dot(hit.normal).max(0.0);
        if direction.y < 0.0 {return (Color::black(),dot);}

        if let Some(_) = self.cast_ray(&ray) {return (Color::black(),dot);}
        else 
        {
            let dot = direction.dot(hit.normal).max(0.0);
            return (self.sample_environment(direction) * dot, dot)
        }
    }


    pub fn calculate_environment_lighting(&self, hit:&Hitresult,samples:i32) -> Color
    {
        let mut out = Color::black();
        let mut accum = 0.0f32;

        for _ in 0..samples {
            let (color,weight) = self.sample_environment_lighting(&hit);
            accum += weight;
            out += color;
        }
        out / accum * hit.node.material.color
    }

    pub fn calculate_environment_lighting_lores(&self, hit:&Hitresult,samples:i32) -> Color {self.calculate_environment_lighting(hit,24)}










	pub fn calc_reflection(&self, hit:&Hitresult, origin:Vector, level:usize) -> Color 
    {
		if level > 2 {return Color::black();}
		//if hit.node.material.glossiness == 1.0 
        {
            let ray = Self::create_reflection_ray(&origin, &hit);
            let new_hit = self.cast_ray(&ray);
			match new_hit {
				Some(new_hit) => {return self.calculate_brdf(&new_hit, hit.pos);},
				None 		=> {return self.sample_environment(ray.dir);}
			}
		} 
//            else {
//				let samples = 64;
//				let mut out = Color::black();
//				for _ in 0..samples {
//					let view = hit.pos - origin;
//					let reflect = Vector::calc_reflect_vector(view, hit.normal);
//					let rand = reflect.random_angle(PI/5.);
//					let ray = Ray::biased_from_direction(hit.pos, rand);
//					match self.cast_ray(&ray) {
//						None => {out = out + self.settings.environment;},
//						Some(new_hit) => {
//							let surface = self.calculate_brdf(&new_hit, hit.pos);
//							out = out + surface;
//						},
//					}
//				}
//				out / samples as f32
//			}
	}

    pub fn create_reflection_ray(origin: &Vector, hit: &Hitresult) -> Ray 
    {
        let view = hit.pos - *origin;
        let reflect_vec = Vector::calc_reflect_vector(view, hit.normal);
		Ray::biased_from_direction(hit.pos, reflect_vec)
    }













	pub fn calc_gi(&self, hit:&Hitresult) -> Color {
		if !self.reads_gi(&hit.node.material) {return Color::black()}
		match self.settings.gi_method_primary {
			GiMethod::BruteForce 		=> self.gi_brute_force2(hit, 0),
			GiMethod::IrradianceMap 	=> self.ir_read(hit.pos),
		}
	}
	pub fn gi_brute_force(&self, hit:&Hitresult, level:i32) -> Color { 
		let passes = (32. / 2.0f32.powi(level)) as usize;
		let mut out_color = Color::black();
		for _ in 0..passes {
			if let Some(new_hit) = self.cast_random_hemispherical_ray(hit.pos, hit.normal) {
				if hit.pos.dist(new_hit.pos) < 50. {
					let color = match self.settings.gi_method_secondary 
					{
						GiMethod::IrradianceMap 	=> self.ir_read(new_hit.pos),
						GiMethod::BruteForce 		=> {
							let mut color = self.approximate_color(&new_hit, hit.pos,true);
							if level < 1 { 
								let secondary_gi = self.gi_brute_force(&new_hit, level+1);
								color += secondary_gi * hit.node.material.color;                // this was unused !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
							}
							color
						},
					};
					let dist = new_hit.pos.dist(hit.pos);
					let falloff = 1. / (dist * 0.1).max(1.);
					out_color = out_color + color;
				}
			}
		}
		out_color * hit.node.material.color / passes as f32
	}
	pub fn gi_brute_force2(&self, hit:&Hitresult, _:i32) -> Color {
		let min = 8;
		let max = 64;
		let mut accum = Color::black();
		let mut samples:Vec<Color> = Vec::with_capacity(max);
		for i in 0..max {
            let random_hemis = hit.normal.random_hemispherical();
            let ray = Ray::biased_from_direction(hit.pos, random_hemis);
			if let Some(new_hit) = self.cast_ray(&ray) 
            {
                let dot = random_hemis.dot(hit.normal).max(0.0);
                //let dist = new_hit.pos.dist(hit.pos);
                //let falloff = 1.0 / (100.0 * dist);
				let color = self.ir_read(new_hit.pos) * dot;

				accum = accum + color;
				samples.push(color);
				if i >= min {
					let mean = accum / (i+1) as f32;
					let v = Renderer::sample_variance(&samples, mean);
					if v < 0.15 { return mean * hit.node.material.color}
				}
			}
		}
		accum * hit.node.material.color / max as f32
	}

	pub fn sample_variance(samples:&Vec<Color>, mean:Color) -> f32 {
		let mut accum = Color::black();
		for s in samples { 
			let x = (*s - mean).pow(2.);
			accum = accum + x;
		}
		let a = accum / (samples.len() - 1) as f32;
		(a.r.powi(2) + a.g.powi(2) + a.b.powi(2)).sqrt()
	}





    // really really slow test
    // render every pixel in serial
    pub fn render_frame_sequential(&self) -> image::RgbaImage 
    {
        let mut img = image::RgbaImage::new(self.image_size.0, self.image_size.1);
        for x in 0..self.image_size.0
        {
            for y in 0..self.image_size.1
            {
                let pixel = self.super_sample_fixed_rate(x as u32,y as u32, 1).to_0_255();
                img.put_pixel(x,y,image::Rgba([pixel.0, pixel.1,pixel.2,255]));
            }
        }
    	img
    }


	fn index_to_xy(&self, i: usize) -> (u32, u32) {
	    let x = i as u32 % self.image_size.0;
	    let y = i as u32 / self.image_size.0;
	    (x,y)
	}

    pub fn render_frame(&self) -> FrameBuffer
    {
	    let mut fb = FrameBuffer::new(self.image_size.0, self.image_size.1);
		
		fb.pixels
		.par_iter_mut()
		.enumerate()
		.for_each( |(index,color)| {
			let (x,y) = self.index_to_xy(index);
			*color = self.super_sample_bokeh(x, y, 1);
		});
		fb

		/*
        for x in 0..self.image_size.0
        {
            for y in 0..self.image_size.1
            {
                let color = self.super_sample_bokeh(x as u32, y as u32, 1);
                fb.set(x,y,color);
            }
        }
    	fb
		*/
    }




	fn spawn_render_progressive_thread(self) -> mpsc::Receiver<image::RgbaImage>
	{
		let (tx,rx) = mpsc::channel();
		let renderer = Arc::new(self);

		// the main renderer thread
	    thread::spawn(move || {

	    	let mut master_framebuffer = FrameBuffer::new(renderer.image_size.0, renderer.image_size.1);
			loop 
			{
	        	let samples = renderer.render_frame();
				master_framebuffer.add_buffer(&samples);
				let image = master_framebuffer.to_image();
				if tx.send(image).is_err()
				{
					return;
				}
			}
	    });
	    rx
	}


	pub fn create_window(&self) -> piston_window::PistonWindow
	{
	    let mut window: piston_window::PistonWindow =
	    piston_window::WindowSettings::new("Frame Buffer", [self.image_size.0, self.image_size.1])
	        .exit_on_esc(true)
	        .build()
	        .unwrap_or_else(|_e| { panic!("Could not create window!")});
	    return window
	}




	// initializes a window
	// starts the interactive render loop
	// awaits updated rgb images to present 
	pub fn start_render(self)
	{

       	let mut window   = self.create_window();
       	let receiver = self.spawn_render_progressive_thread();

       	let mut last_frame : Option<piston_window::G2dTexture> = None;

        println!("starting loop");
        while let Some(e) = window.next()
        {
        	// waiting for our next bitmap image
            if let Ok(img) = receiver.try_recv()
            {
                let tex = piston_window::Texture::from_image(
                    &mut window.create_texture_context(),
                    &img,
                    &piston_window::TextureSettings::new())
                    .unwrap();
                last_frame = Some(tex);

            }

            // if we don't have one check if we have an old one and draw that
            if let Some(ref tex) = last_frame
            {
                window.draw_2d(&e, |c, g, _| {
                    piston_window::clear([1.0; 4], g);
                    piston_window::image(tex, c.transform, g)
                });
            }
        }
	}
	

	
	
}

