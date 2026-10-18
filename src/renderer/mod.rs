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
use rand::Rng;
use rayon::prelude::*;

use vectormath::*;
use color::*;
use camera::*;
use geometry::*;
use material::*;
use framebuffer::FrameBuffer;






const PI: f32 		= 3.14159265359; 
const TWO_PI: f32 	= 6.2831832; 




#[derive(Eq,PartialEq)]  
pub enum GiMethod {
    BruteForce,
    IrradianceMap
}


pub enum RenderMode {
    Beauty,
    Debug_IrradianceMap,
    Debug_IrradianceMap_Samples,
    Debug_GI,
    Debug_Normals,
    Debug_UVs,
	Debug_Reflections,
}




pub struct Settings {
	render_mode: RenderMode,
    environment:Color,
    environment_map: Option<Texture>,
    reflections_enabled: bool,
    shadows_enabled: bool,
    gi_enabled: bool,
    gi_method_secondary: GiMethod,
}

impl Settings {
    fn new() -> Settings {
        Settings { 
			render_mode: RenderMode::Beauty,
            shadows_enabled:true, 
            reflections_enabled: true,
            environment: Color::black(),
            environment_map: None,
            gi_method_secondary: GiMethod::IrradianceMap,
            gi_enabled: true,
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
    pos:         Vector,
	origin_pos:  Vector,
    normal:      Vector,
    id:          u32,
    node:        &'a Geo,
    uv:          Vec2,
    tangent:     Vector,
	material_id: usize
}
impl<'a> Hitresult<'a> {
    pub fn new(
		pos: Vector,
		normal: Vector,
		origin_pos: Vector,
		uv:Vec2,
		obj:u32,
		node: &Geo,
		material_id: usize
	) -> Hitresult
	{
        Hitresult{pos, normal, origin_pos, id: obj, node, uv, tangent:Vector::new(0.0,0.0,0.0), material_id}
    }
}













pub struct Renderer {
	pub camera: Camera,
	nodes: Vec<Geo>,
	materials : Vec<Material>,
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
		let mut cam = Camera::new(x,y);

		let mut r = Renderer {
			camera: cam, 
			nodes: nodes, 
			materials :     Vec::new(),
			lights:         Vec::new(), 
			image_size:     (x,y),
			settings:       Settings::new(),
			irradiance_map: IRClass::new(),
		};
		r
	}


	// ambigious TODO
	pub fn add(&mut self, mut g:Geo) { 
		self.nodes.push(g);
	}

	pub fn add_material(&mut self, mtl:Material) -> usize
	{
		let index = self.materials.len();
		self.materials.push(mtl);
		index
	}




	pub fn create_demo_scene(&mut self) 
	{
		
		self.camera.rotate_x(-0.5);
		self.camera.pan_vertical(2.0);
		self.camera.walk(-20.0);
		//self.camera.set_focal_length(1.0);
		//

		let blender_cam: [[f32; 4]; 4] = [
			[0.9253, -0.1665,  0.3408,  3.1437],
			[0.3793,  0.4062, -0.8313, -5.8090],
			[0.0000,  0.8985,  0.4390,  4.0019],
			[0.0000,  0.0000,  0.0000,  1.0000]
			];
			
		self.camera.from_blender_transform(&blender_cam);



		let grey_mtl = Material::new();

		let red_mtl = Material::new()
			.set_reflection_enabled(true)
			.set_ior(1.6)
			.set_roughness(0.0)
			.set_color(Color::red());

		let chrome_mtl = Material::new()
			//.set_color(Color::black())
			.set_reflection_enabled(false)
			.set_ior(1.2)
			.set_roughness(0.0);
		
		//mg.material.texmap_diffuse = Some(tex);
		//mg.material.texmap_normal = Some(nmap);
		
		let id_red_mtl  = self.add_material(red_mtl);
		let id_grey_mtl = self.add_material(grey_mtl);
		let id_chrome_mtl = self.add_material(chrome_mtl);

		

		//self.settings.render_mode=RenderMode::Debug_IrradianceMap;
		//self.settings.render_mode=RenderMode::Debug_IrradianceMap_Samples;
		//self.settings.render_mode=RenderMode::Debug_GI;
		//self.settings.render_mode=RenderMode::Debug_Reflections;
		
		//self.settings.render_mode=RenderMode::Debug_Normals;
		self.settings.gi_enabled = false;
		
		
		let mut project_root = Path::new(env!("CARGO_MANIFEST_DIR"));

		//let hdri_path = project_root.join("models/tomoco_studio.exr");
		let hdri_path = project_root.join("models/HdrSkySunset011_HDR_HIRES.exr");
        let hdri = material::Texture::open_exr(hdri_path.to_str().unwrap());
        self.settings.environment_map = Some(hdri);


		
		let mut s = Geo::new()
			.translate(Vector::new(0.0, 0.5, 0.0));
		s.shape = Shape::Sphere(1.);
		s.materials.push(Some(id_red_mtl));
		//self.add(s);


		//let stage_path = project_root.join("models/monkey.obj");
		let stage_path = project_root.join("models/blender_test.obj");
		//let stage_path = project_root.join("models/stage.obj");

		let mut stage_mesh = open(stage_path.to_str().unwrap()).unwrap();
		
		let mut stage_geo = Geo::from_mesh(stage_mesh)
			.assign_material(0,id_grey_mtl)
			.assign_material(1,id_red_mtl);

        self.add(stage_geo);

		/*
		// pot
		//let pot_path = project_root.join("models/teapot.obj");
		let pot_path = project_root.join("models/skull.obj");
		//let pot_path = project_root.join("models/sphere.obj");
		let mut mesh = open(pot_path.to_str().unwrap()).unwrap()
			.translate_y(0.5)
			.rotate_y(3.141);
			
		let mut mg = Geo::from_mesh(mesh)
			.assign_material(0,id_red_mtl);
		//self.add(mg);
		*/
		


		let mut s2 = Geo::new();
		s2.shape = Shape::Sphere(12.);
		s2.materials.push(Some(id_chrome_mtl));
		//self.add(s2);


		let mut l = Light::new();
		l.pos.y = 10.;
		l.pos.x = -10.;
		l.pos.z = -35.;
		l.color = Color::new(0.85,0.85,1.0);
        l.color =  l.color * 3.0;
		//self.lights.push(l);

		let mut l2 = Light::new();
		l2.pos.y = 5.;
		l2.pos.x = 15.;
		l2.pos.z = -45.;
		l2.color = Color::new(1.,1.,1.);
        l2.color =  l2.color * 3.0;
		//self.lights.push(l2);
	}
	
	fn warmup(&mut self){
		// call the prepass on every nodes mesh
		for node in &mut self.nodes 
		{
			if let Shape::Mesh(ref mut mesh) = node.shape {mesh.prepass();}
		}
		// create ir
		if self.settings.gi_enabled && self.settings.gi_method_secondary == GiMethod::IrradianceMap {
			println!("Calculation Irradiance Map");
			self.create_ir();
		}
		
	}


	// hit result contains the slot number
	// we have to get the material identifier from the geo object
	// which we then can use to get the material from the renderer
	pub fn get_material<'a>(&'a self, hit: &Hitresult) -> &'a Material {
		let slot       = hit.material_id;
		let identifier = hit.node.materials[slot].unwrap();
		&self.materials[identifier]
	}




	// save on calculation for objects that don't care about light
	// ie light materials
	pub fn reads_gi(&self, m:&Material) -> bool {
		if m.color == Color::black() {return false}
		match m.diffuse {
			BRDF::None     => false,
			BRDF::Constant => false,
			_ => true,
		}
	}

	pub fn create_ir(&mut self) {
		let samples = 1500;
		let max_depth = 2;
		let mut i = 0;
		let mut ir = IRClass::new();
		let camera_pos = self.camera.get_pos();
		for _ in 0..samples {
			if let Some(hit) = self.cast_random_eye_ray() {
				match ir.check_sample(&hit) {
					Some(_) => (),
					None =>  {
						let mut out = self.approximate_direct_diffuse_light(&hit, camera_pos);
						let material = self.get_material(&hit);
						if self.reads_gi(material) {out += self.ir_secondaray(&hit, &mut ir, max_depth);}
						ir.add(hit.pos, out, hit.normal);
					}
				}
			}
		}
		self.irradiance_map = ir;
	}
    

	pub fn ir_secondaray(&self, hit:&Hitresult, ir:&mut IRClass, level:usize) -> Color {
		let mut out = Color::black();
		let samples = 16;
		let material = self.get_material(hit);
		let mut rng = rand::rng();

		for _ in 0..samples
		{
        	let direction = Vector::cosine_sample_hemisphere(hit.normal, &mut rng);
			let ray = Ray::biased_from_direction(hit.pos, direction);

			if let Some(new_hit) = self.cast_ray(&ray) {
				let c = match ir.check_sample(&new_hit) {
					Some(col) => col,
					None =>  {
						let mut surface = self.approximate_direct_diffuse_light(&new_hit, hit.pos);
						let next_material = self.get_material(&new_hit);
						if level > 0 && self.reads_gi(material)
						{
							let gi = self.ir_secondaray(&new_hit, ir, level-1) * next_material.color;
							surface = surface + gi;
						}
						if level > 0 { ir.add(new_hit.pos, surface, new_hit.normal);}
						surface
					}
				};
				out = out + c;
			}
		}
		(out / samples as f32) * material.color
	}


	pub fn ir_read(&self, pos:Vector) -> Color 
	{ 
		let near_points = self.irradiance_map.near_points(pos, 20);
		if near_points.len() == 0 {panic!("empty ir map");}

		let mut out = Color::black();
		let mut accum = 0.0;

		for s in near_points {
			let (_, &(col, _sample_pos)) = s;

			let dist = pos.dist(_sample_pos);
			let denom = 1.0/dist.sqrt().max(0.0001) as f32;
			out = out + col * denom;
			accum += denom;
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
	





	pub fn ray_from_pixel(&self, x: f32, y: f32) -> Ray{
		let dir = self.camera.calc_ray_direction(x,y);
		Ray::from_direction(self.camera.get_pos(), dir)
	}

	// for irradiance map precalculations
	pub fn cast_random_eye_ray(&self) -> Option<Hitresult> {
		let x = rand::random::<f32>() * self.image_size.0 as f32;
		let y = rand::random::<f32>() * self.image_size.1 as f32;
		let ray = self.ray_from_pixel(x,y);
		self.cast_ray(&ray)
	}
	/*
	pub fn cast_random_hemispherical_ray(&self, p:Vector, n:Vector) -> Option<Hitresult> {
		let hemis = n.random_hemispherical();
		let ray = Ray::biased_from_direction(p, hemis);
		self.cast_ray(&ray)
	}
	*/

/*
	pub fn calc_refraction(&self, hit:&Hitresult, origin:Vector) -> Color {
		fn calc_refraction_secondary(r:&Renderer, hit:&Hitresult, origin:Vector, level:usize) -> Color {
			let i = (hit.pos - origin).norm();
			let dir = Vector::calc_refract_vector(i, hit.normal, 1.3);
			let ray = Ray::biased_from_direction(hit.pos, dir);
			if let Some(new_hit) = r.cast_ray(&ray) {
				if level > 2 {return r.approximate_color(&new_hit, hit.pos,false);}
				else if new_hit.node.material.refraction_enabled {
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
*/



   

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
				let color = self.calculate_brdf(&hit, self.camera.get_pos());
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
		let material = self.get_material(hit);

		let mut out = Color::black();
			for light in &self.lights
			{
				if self.settings.shadows_enabled && self.check_shadow(hit.pos, &light) {continue;}
				let diffuse = material.calc_diffuse(hit.pos, hit.normal, hit.tangent, &light, origin, hit.uv);
				//let spec =    material.calc_specular(hit.pos, hit.normal, &light, origin);
				out = out + diffuse;
			}
		return out;
	}


	// simplified function for GI sampling
	pub fn approximate_direct_diffuse_light(&self, hit:&Hitresult, origin:Vector) -> Color
	{
		let mut diffuse = Color::black();
		let material = self.get_material(hit);

		for light in &self.lights
		{
			if self.settings.shadows_enabled && self.check_shadow(hit.pos, &light) {continue;}
			diffuse = diffuse + material.calc_diffuse(hit.pos, hit.normal, hit.tangent, &light, origin, hit.uv);
		}
		let env = self.calculate_environment_lighting(&hit,24) * material.color;
		diffuse + env
    }


	pub fn approximate_color(&self, hit:&Hitresult, origin:Vector, reflection: bool) -> Color 
    {
		let material = self.get_material(hit);
		if reflection && material.reflection_enabled 
        {
			let fresnel = Self::calc_fresnel(hit.pos, origin, hit.normal, material.ior);
            let ray = Self::create_reflection_ray(origin, hit.pos, hit.normal);
            let hit_opt = self.cast_ray(&ray);

			let reflect = match hit_opt {
				Some(new_hit) => {self.approximate_color(&new_hit, hit.pos, false)},
				None 		=> {self.settings.environment}
			};
			let surface = self.calculate_brdf(&hit, origin) * (1. - fresnel);
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
	
	// shift a f32 sample randomly by (0-1)
	/*
	pub fn jitter(base:f32) -> f32 {
		let shift = rand::random::<f32>();
		(base+shift).fract()
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

/*
	pub fn super_sample_bokeh(&self, x: u32, y:u32, sample_count:u32) -> Color {
		let focal_dist = 40.0;
		let mut out = Color::black();
		let r = 1.2;

		let up = self.camera.up;
		let right = self.camera.dir.cross(up);

		let old_dir = {
			self.camera.calc_ray_direction(x as f32, y as f32)
		};
		let focal_point = self.camera.get_pos() + old_dir * focal_dist;


		for i in 0..sample_count {
			let a = i as f32 + rand::random::<f32>();
			let u = (a * TWO_PI / sample_count as f32 ).cos() * r;
			let v = (a * TWO_PI / sample_count as f32 ).sin() * r;

			let origin = self.camera.get_pos() + right * u + up * v;

			let dir = (focal_point - origin).norm();
			let ray = Ray::from_direction(origin, dir);

			out += self.cast_primary_ray(&ray);
		}
		out / sample_count as f32
	}
*/






	// TODO move the beauty part in it's own function
	// and clean up / rename all the indirect sampling functions

	pub fn cast_primary_ray(&self, ray:&Ray) -> Color {
		let hit_result = self.cast_ray(ray);
		match hit_result {
			None      => self.sample_environment(ray.dir),
			Some(hit) => {
				match self.settings.render_mode{
					RenderMode::Debug_Normals           => hit.normal.normal_to_color(),
					RenderMode::Debug_UVs               => hit.uv.to_color(),
					RenderMode::Debug_IrradianceMap     => self.ir_read(hit.pos),
					RenderMode::Debug_IrradianceMap_Samples   => self.ir_render_samples(&hit), 
					RenderMode::Debug_GI                => self.calc_gi(&hit),
					RenderMode::Beauty 					=> self.calc_principled_shading(&hit),
					RenderMode::Debug_Reflections 		=> self.debug_raw_reflections(&hit),
				}
			}
		}
	}
	

	pub fn calc_fresnel(pos:Vector, view_origin:Vector, normal:Vector, ior:f32) -> f32
	{
		// ior to r0
		let r0 = ((ior - 1.0) / (ior + 1.0)).powi(2);

		let v = (view_origin - pos).norm();
		let cos_theta = v.dot(normal).max(0.0);
		let f = r0 + (1.0-r0) * (1.0-cos_theta).powf(5.0);
		f.clamp(0.0, 1.0)
	}
	

	pub fn debug_raw_reflections(&self, hit:&Hitresult) -> Color{
		let material = self.get_material(hit);
		if material.reflection_enabled
		{
			let fresnel = Self::calc_fresnel(hit.pos, self.camera.get_pos(), hit.normal, material.ior);
			let reflect = self.calc_reflection(material, hit.normal, hit.pos, hit.origin_pos, 0);
			return reflect;
		} else{return Color::black();}
	}

	pub fn calc_principled_shading(&self, hit:&Hitresult) -> Color
	{
		//let view = hit.pos - self.camera.pos;
		//let reflect = Vector::calc_reflect_vector(view, hit.normal);
		//return reflect.to_color();
		//return hit.normal.random_hemispherical().normal_to_color();
		//return self.calculate_environment_lighting(&hit,24);
		let material = self.get_material(hit);
		let mut surface = self.calculate_brdf(&hit, hit.origin_pos);
		let env = self.calculate_environment_lighting(&hit,24);
		surface += env * material.color;

		let mut out = if material.reflection_enabled {
			let fresnel = Self::calc_fresnel(hit.pos, hit.origin_pos, hit.normal, material.ior);
			let reflect = self.calc_reflection(material, hit.normal, hit.pos, hit.origin_pos, 0);
			(surface * (1.0 - fresnel)) + (reflect * fresnel)
		} else{
			surface
		}; 
		if self.settings.gi_enabled {out = out + self.calc_gi(&hit) * material.color;} 

		out
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
	
	// cast ray into direction
	// return option with distance
	/*
	pub fn cast_shadow_ray(&self, sample_pos:Vector, direction:Vector) -> Option<f32>
	{
		let ray = Ray::new(sample_pos, direction);
		let hit = self.cast_ray(&ray);
		if let Some(Hitresult{pos:hitpos,..}) = hit Some(hitpos.distance(pos))
		else None
	}
	*/





	// sample the current environment from vector
    fn sample_environment(&self, r: Vector) -> Color {

        //return Color::new(0.2,0.3,0.6).interpolate(Color::new(0.4,0.3,0.2), r.y);

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



    pub fn calculate_environment_lighting(&self, hit:&Hitresult, samples:u16) -> Color
    {
        let mut out = Color::black();
		let mut rng = rand::rng();

		// cosine samples don't use the dot product (lambert)
		// this will keep the accumulation unbiased
        for _ in 0..samples
		{
        	let direction = Vector::cosine_sample_hemisphere(hit.normal, &mut rng);
			// check for occlusion
			let ray = Ray::biased_from_direction(hit.pos, direction);
			if let Some(_) = self.cast_ray(&ray) {continue;}

            let color = self.sample_environment(direction);
            out += color;
        }
        out/samples as f32
    }





	// return the sample direction vector 
	// and the PDF
	// v is the vector pointing from the geo to the camera
	// probably needds inverting
	fn sample_ggx(v:Vector, n:Vector, roughness:f32, u:Vec2) -> (Vector, f32)
	{
		let alpha = roughness * roughness;// * roughness * roughness;
		
		let phi = 2.0 * PI * u.x;
		let cosTheta = ((1.0 - u.y) / (1.0 + (alpha - 1.0) * u.y)).sqrt();
		let sinTheta = ((1.0 - cosTheta * cosTheta).max(0.0)).sqrt();
		
		let h_tangent = Vector::new(
			sinTheta * phi.cos(),
			sinTheta * phi.sin(),
			cosTheta
		);
		
		let up = if n.z.abs()<0.999 {Vector::new(0.,0.,1.)} else {Vector::new(1.,0.,0.)};
		let tangent = up.cross(n).norm();
		let bitangent = n.cross(tangent);
		let h = (tangent * h_tangent.x + bitangent * h_tangent.y + n * h_tangent.z).norm();

		// our direction vector
		let L = (h * 2.0 * v.dot(h) - v).norm();
		
		// calculuate pdf
		let dotNH = n.dot(h).max(0.0);
		let dotVH = v.dot(h).max(0.0);
		
		let denom = dotNH * dotNH *(alpha - 1.0) + 1.0;
		let d = alpha / (PI * denom * denom);
		let pdf = (d *dotNH) / (4.0 * dotVH);

		// (direction, pdf)	
		(L,pdf)
	}






	pub fn calc_reflection(&self, material:&Material, normal:Vector, current_pos: Vector, origin:Vector, level:usize) -> Color 
    {
		let roughness = material.roughness;
		if level > 2 {return Color::black();}
		
		if roughness == 0.0 
        {
            let ray = Self::create_reflection_ray(origin, current_pos,normal);
            let new_hit = self.cast_ray(&ray);
			match new_hit {
				Some(new_hit) => {return self.calculate_brdf(&new_hit, current_pos);},
				None 		=>   {return self.sample_environment(ray.dir);}
			}
		} 
		
        else 
		{
			let samples = 64;
			let mut accum = Color::black();
			for i in 0..samples
			{
				let view = (origin - current_pos).norm(); // ggx needs the vector to point to the viewer
				let shift_x = rand::random::<f32>();
				let shift_y = rand::random::<f32>();
				
				let mut u = Vec2::halton_sample(i+1);
				u = Vec2::new(
					(u.x + shift_x).fract(),
					(u.y + shift_y).fract()
				);
				// taking a shortcut here. there should be more weighting functions and such
				// TODO
				let (direction,_pdf) = Self::sample_ggx(view, normal, roughness, u);
				let ray = Ray::biased_from_direction(current_pos, direction);

				match self.cast_ray(&ray) {
					None =>
						{accum = accum + self.sample_environment(ray.dir);}
					Some(new_hit) => {
						let surface = self.calculate_brdf(&new_hit, current_pos);
						accum = accum + surface;
					},
				}
			}
			accum / samples as f32
		}
	}

    pub fn create_reflection_ray(origin: Vector, hit_pos: Vector, normal: Vector) -> Ray 
    {
        let view = hit_pos - origin;
        let reflect_vec = Vector::calc_reflect_vector(view, normal);
		Ray::biased_from_direction(hit_pos, reflect_vec)
    }











	// returns the raw indirect illumination
	// NOT yet multiplied by surface color
	// how does this work with irm tho?
	
	pub fn calc_gi(&self, hit:&Hitresult) -> Color {
		let material = self.get_material(hit);
		if !self.reads_gi(material) {return Color::black()}
		self.gi_brute_force2(hit, 0)
	}
	

/*
	// unused atm
	// only bruteforce->IRM atm
	// TODO merge into one
	pub fn gi_brute_force(&self, hit:&Hitresult, level:i32) -> Color { 
		let passes = (32. / 2.0f32.powi(level)) as usize;
		let mut out_color = Color::black();
		let material = self.get_material(hit);
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
								color += secondary_gi * material.color;
							}
							color
						},
					};
					//let dist = new_hit.pos.dist(hit.pos);
					//let falloff = 1. / (dist * 0.1).max(1.);
					out_color = out_color + color;
				}
			}
		}
		out_color / passes as f32
	}
*/


	pub fn gi_brute_force2(&self, hit:&Hitresult, _:i32) -> Color {
		let min = 4;
		let max = 32;
		let threshold = 0.1;
		let mut rng = rand::rng();

		let mut accum = Color::black();
		let mut samples:Vec<Color> = Vec::with_capacity(max);
		for i in 0..max {
            let direction = Vector::cosine_sample_hemisphere(hit.normal, &mut rng);
            let ray = Ray::biased_from_direction(hit.pos, direction);
			if let Some(new_hit) = self.cast_ray(&ray) 
            {
				let dist = new_hit.pos.dist(hit.pos);
				let _falloff = 1. / (dist * 0.1).max(1.);
				let color = self.ir_read(new_hit.pos) * _falloff;

				accum = accum + color;
				samples.push(color);
				if i >= min {
					let mean = accum / (i+1) as f32;
					let v = Renderer::sample_variance(&samples, mean);
					if v < threshold { return mean;}
				}
			}
		}
		// no threshold reached
		// used all max samples
		accum  / max as f32
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
    fn render_frame_sequential(&self) -> image::RgbaImage 
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
			*color = self.super_sample_fixed_rate(x, y, 1);
			//*color = self.super_sample_bokeh(x, y, 1);
		});
		fb
    }




	fn spawn_render_progressive_thread(self) -> mpsc::Receiver<image::RgbaImage>
	{
		let (tx,rx) = mpsc::channel();
		let renderer = Arc::new(self);

		// the main renderer thread
	    thread::spawn(move || {

	    	let mut master_framebuffer = FrameBuffer::new(renderer.image_size.0, renderer.image_size.1);
			let mut counter = 1u32;
			
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
	pub fn start_render(mut self)
	{

		self.warmup();

       	let mut window   = self.create_window();
       	let receiver = self.spawn_render_progressive_thread();

       	let mut last_frame : Option<piston_window::G2dTexture> = None;

        println!("Starting Render");
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

