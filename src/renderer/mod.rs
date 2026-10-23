#![allow(warnings)] 


pub mod vectormath;
pub mod camera;
pub mod geometry;
pub mod material;
pub mod color;
pub mod framebuffer;
pub mod scene;

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
use scene::SceneImporter;





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
	Debug_Raw_GI,
    Debug_GI,
    Debug_Normals,
    Debug_UVs,
	Debug_Reflections,
	Debug_Focus,
}




pub struct Settings {
	render_mode: RenderMode,
    environment:Color,
    environment_map: Option<Texture>,
	environment_map_multiplier : f32,
	environment_map_rotation : f32,
    reflections_enabled: bool,
    shadows_enabled: bool,
    gi_enabled: bool,
    gi_method_secondary: GiMethod,
	irradiance_map_primary_samples: usize,
	dof_enabled: bool,
}

impl Settings {
    fn new() -> Settings {
        Settings { 
			render_mode: RenderMode::Beauty,
            shadows_enabled:true, 
            reflections_enabled: true,
            environment: Color::black(),
            environment_map: None,
			environment_map_multiplier: 1.0,
			environment_map_rotation: 0.0,
            gi_method_secondary: GiMethod::IrradianceMap,
            gi_enabled: true,
			irradiance_map_primary_samples: 1500, // unused atm
			dof_enabled: false,
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
    fn add(&mut self, position:Vector, color:Color, normal:Vector) {
        self.samples.add(position.to_array(), (color,normal)).unwrap();
    } 
	// used during creation
	// check if this sample is close enough
	// and if the normal is close enough
	// if not return None
    fn _check_sample(&self, hit:&Hitresult)  -> Option<Color> 
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
    fn check_sample(&self, hit: &Hitresult) -> Option<Color> {
		let near = self.near_points(hit.pos, 8);
		let max_dist = 0.1;
		let max_angle = 0.1;
		for (d, &(c, n)) in near {
			if d > max_dist*max_dist {
				continue;
			}
			if n.angle_between(hit.normal) > max_angle {
				continue;
			}
			return Some(c);
		}
		None
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
	environment_cdf : Option<EnvironmentCdf>
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
			environment_cdf: None,
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
	
	pub fn set_environment(&mut self, environment: Texture)
	{
		self.environment_cdf = Some(EnvironmentCdf::new(&environment));
		self.settings.environment_map = Some(environment);
	}

	pub fn set_image_size(&mut self, w:u32, h:u32) {
		self.image_size.0 = w;
		self.image_size.1 = h;
		self.camera.image_width = w;
		self.camera.image_height = h;
		let ratio = w as f32 / h as f32;
		self.camera.ratio = ratio;
	}





	pub fn create_demo_scene(&mut self) 
	{


		let mut project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
		//let scene_path = project_root.join("models/blender_scene.toml");
		let scene_path = project_root.join("models/scene_round_room.toml");


		SceneImporter::import_scene(self, scene_path);

		let mut project_root = Path::new(env!("CARGO_MANIFEST_DIR"));

		//let hdri_path = project_root.join("models/tomoco_studio.exr");
		//let hdri_path = project_root.join("models/HdrSkySunset011_HDR_HIRES.exr");
        //let hdri = material::Texture::open_exr(hdri_path.to_str().unwrap());
        //self.settings.environment_map = Some(hdri);
		return;
	
		




		
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
		//self.settings.shadows_enabled = false;
		//self.settings.gi_enabled = false;
		
		


		
		let mut s = Geo::new()
			.translate(Vector::new(0.0, 0.5, 0.0));
		s.shape = Shape::Sphere(1.);
		s.materials.push(Some(id_red_mtl));
		//self.add(s);


		//let stage_path = project_root.join("models/monkey.obj");
		let stage_path = project_root.join("models/blender_test.obj");
		//let stage_path = project_root.join("models/stage.obj");

		let mut stage_mesh = OBJ_Importer::open(stage_path.to_str().unwrap()).unwrap();
		
		let mut stage_geo = Geo::from_mesh(stage_mesh)
			.assign_material(0,id_red_mtl)
			.assign_material(1,id_grey_mtl);

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
		// create ir only if we need it
		if self.settings.gi_enabled && self.settings.gi_method_secondary == GiMethod::IrradianceMap
		{
			match self.settings.render_mode{
                RenderMode::Beauty|RenderMode::Debug_GI|RenderMode::Debug_Raw_GI|RenderMode::Debug_IrradianceMap|RenderMode::Debug_IrradianceMap_Samples => {
					println!("Calculating Irradiance Map");
					self.create_ir();
				},
                _   => ()
            }
		}
	}

	fn calculate_falloff(distance: f32) -> f32 {
		// prevent dividing by zero if distance is 0
		let epsilon = 0.0001; 
		(distance * distance + epsilon).min(1.0)
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
		let samples = 3000;
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
				let dist = new_hit.pos.dist(hit.pos);
				let falloff = Self::calculate_falloff(dist);
				out = out + c*falloff;
			}
		}
		(out / samples as f32) * material.color
	}



	pub fn ir_read(&self, pos: Vector, normal: Vector) -> Color {
		let near_points = self.irradiance_map.near_points(pos, 20);

		if near_points.is_empty() {
			return Color::black();
		}

		let mut out = Color::black();
		let mut accum = 0.0_f32;

		let sigma = 0.35_f32;
		let normal_power = 4.0_f32;
		let epsilon = 1e-6_f32;

		for (dist, &(col, sample_normal)) in near_points {
			// kdtree returns the squared distance, so we need to sqrt()
			let d = dist.sqrt() as f32;

			// Gaussian spatial falloff.
			let distance_weight = (-0.5 * (d / sigma).powi(2)).exp();

			// Penalize samples from differently oriented surfaces.
			// Assumes both normals are normalized.
			let normal_dot = normal.dot(sample_normal).clamp(0.0, 1.0);
			let normal_weight = normal_dot.powf(normal_power);

			let weight = distance_weight * normal_weight;

			out = out + col * weight;
			accum += weight;
		}

		if accum <= epsilon {
			return Color::black();
		}

		out / accum
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

	pub fn calculate_diffuse(&self, hit:&Hitresult, environment_samples:u16) -> Color {
		let material = self.get_material(hit);
		let mut out = Color::black();
			for light in &self.lights
			{
				if self.settings.shadows_enabled && self.check_shadow(hit.pos, &light) {continue;}
				let diffuse = material.calc_diffuse(hit.pos, hit.normal, hit.tangent, &light, hit.origin_pos, hit.uv);
				out = out + diffuse;
			}
		let env = self.calculate_environment_lighting(&hit, environment_samples).clamp(3.0); // clamp direct environment light at 3
		out += env * material.color;
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
		let env = self.calculate_environment_lighting(&hit,16) * material.color;
		diffuse + env
    }
	


	pub fn super_sample_fixed_rate(&self, x: u32, y:u32, samples:u32) -> Color {
		let mut rng = rand::rng(); 	// -> ThreadRng
		let mut out = Color::black();
		for s in 0..samples
		{
			let halton = Vec2::halton_sample(s as i32);
			let mut ray = self.ray_from_pixel(x as f32 + halton.x ,y as f32 + halton.y);
			let c = self.cast_primary_ray(&ray, &mut rng);
			out += c;
		}
		out / samples as f32
	}


	pub fn super_sample_defocus(&self, x: u32, y:u32, sample_count:u32) -> Color {
		
		/*
		let random_in_unit_disk = |i| {
			let mut seed = i;
			loop {
			let mut v = Vec2::halton_sample(seed);
			v = v * 2.0 - Vec2::new(1.0,1.0);
			if v.length_squared() < 1.0 {return v}
			else {seed += 1234;}
			}
		};
		*/
		let random_in_unit_disk = |i| {
			let s = Vec2::halton_sample(i);
			let u = (s.x * TWO_PI ).cos();
			let v = (s.x * TWO_PI ).sin();
			Vec2::new(u,v) * s.y * 2.0

		};
				

		

		let mut out = Color::black();
		let mut rng = rand::rng(); 	// -> ThreadRng

		for i in 0..sample_count
		{
			let o = rand::random::<i32>();
			let sample_pos = random_in_unit_disk(o+i as i32);
			let ray = self.camera.get_ray_defocus(x as f32, y as f32, sample_pos);

			out += self.cast_primary_ray(&ray, &mut rng);
		}
		out / sample_count as f32
	}







	// TODO move the beauty part in it's own function
	// and clean up / rename all the indirect sampling functions

	pub fn cast_primary_ray(&self, ray:&Ray, rng: &mut impl Rng) -> Color {
		let hit_result = self.cast_ray(ray);
		match hit_result {
			None      => self.sample_environment(ray.dir),
			Some(hit) => {
				match self.settings.render_mode{
					RenderMode::Debug_Normals           => hit.normal.normal_to_color(),
					RenderMode::Debug_UVs               => hit.uv.to_color(),
					RenderMode::Debug_IrradianceMap     => self.ir_read(hit.pos,hit.normal),
					RenderMode::Debug_IrradianceMap_Samples   => self.ir_render_samples(&hit), 
					RenderMode::Debug_GI                => self.debug_gi(&hit),
					RenderMode::Debug_Raw_GI 			=> self.calc_gi(&hit,8,24),
					RenderMode::Beauty 					=> self.calc_principled_shading(&hit, rng),
					RenderMode::Debug_Reflections 		=> self.debug_raw_reflections(&hit),
					RenderMode::Debug_Focus 		    => self.debug_focus(&hit),
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
	
	fn debug_gi(&self, hit:&Hitresult) -> Color{
		let material = self.get_material(hit);
		self.calc_gi(&hit,8,24) * material.color
	}

	fn debug_focus(&self, hit:&Hitresult) -> Color{
		let dist = hit.pos.dist(hit.origin_pos);
		let x = self.camera.focus_dist - dist;
		if x.abs() < 0.1 {return Color::new(0.0,1.0,0.0);}
		if (x>0.0) {Color::new(1.0,0.0,0.0)}
		else {Color::new(0.0,0.0,1.0)}
		
	}

	fn debug_raw_reflections(&self, hit:&Hitresult) -> Color{
		let material = self.get_material(hit);
		//if material.reflection_enabled
		//{
			//let fresnel = Self::calc_fresnel(hit.pos, self.camera.get_pos(), hit.normal, material.ior);
			let reflect = self.calc_reflection(material, hit.normal, hit.pos, hit.origin_pos, 0);
			return reflect;
		//} else{return Color::black();}
	}




	pub fn calc_principled_shading(&self, hit:&Hitresult, rng: &mut impl Rng) -> Color
	{
		let material = self.get_material(hit);
		let mut surface = self.calculate_diffuse(&hit,64);

		let mut out = if material.reflection_enabled {
			let fresnel = Self::calc_fresnel(hit.pos, hit.origin_pos, hit.normal, material.ior);
			let reflect = self.calc_reflection(material, hit.normal, hit.pos, hit.origin_pos, 0);
			(surface * (1.0 - fresnel)) + (reflect * fresnel)
		} else{
			surface
		}; 
		if self.settings.gi_enabled {out = out + self.calc_gi(&hit,8,32) * material.color;} 
		out.clamp(5.0)
	}


	// cheaper function for secondary rays - reflections and refractions
	pub fn calc_secondary_shading(&self, hit:&Hitresult) -> Color
	{
		let gi_clamp = 2.0;
		let material = self.get_material(hit);
		let mut out = self.calculate_diffuse(&hit,8);
		/*
		let mut out = if material.reflection_enabled {
			let fresnel = Self::calc_fresnel(hit.pos, hit.origin_pos, hit.normal, material.ior);
			let reflect = self.calc_reflection(material, hit.normal, hit.pos, hit.origin_pos, 0);
			(surface * (1.0 - fresnel)) + (reflect * fresnel)
		} else{
			surface
		}; 
		*/
		//if self.settings.gi_enabled {out = out + self.calc_gi(&hit,4,4).clamp(gi_clamp) * material.color;} 
		if self.settings.gi_enabled {out = out + self.ir_read(hit.pos,hit.normal).clamp(gi_clamp);}
		out.clamp(5.0)
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
		if let Some(hdri) = &self.settings.environment_map {
			let r = r.norm();
			let u = (r.x.atan2(r.z) / TWO_PI + 0.5 + self.settings.environment_map_rotation).fract();
			let v =  (r.y.acos() / PI); 
			return hdri.get_pixel(u, v) * self.settings.environment_map_multiplier;
		} else {
			self.settings.environment
		}
	}






	// raw environment light
	// not multiplied by albedo

	// use cosine
    pub fn _calculate_environment_lighting(&self, hit:&Hitresult, samples:u16) -> Color
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

	// use cdf
    pub fn calculate_environment_lighting(&self, hit: &Hitresult, samples: u16) -> Color
	{
		if samples == 0 {
			return Color::black();
		}

		let mut out = Color::black();
		let mut rng = rand::rng();

		if let (Some(cdf), Some(env)) = (
			&self.environment_cdf,
			&self.settings.environment_map,
		) {
			for _ in 0..samples {
				let sample = cdf.sample(env, self.settings.environment_map_rotation, &mut rng);
				let cos_theta = hit.normal.dot(sample.direction).max(0.0);

				if cos_theta <= 0.0 || sample.pdf <= 0.0 {
					continue;
				}
				let ray = Ray::biased_from_direction(hit.pos, sample.direction);
				if self.cast_ray(&ray).is_some() {
					continue;
				}
				out += sample.radiance
					* (cos_theta / (PI * sample.pdf)) * self.settings.environment_map_multiplier;
			}
		}
		out / samples as f32
	}




	//ggx

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
		if level > 2 {return Color::red();}
		
		if roughness == 0.0 
        {
            let ray = Self::create_reflection_ray(origin, current_pos,normal);
            let new_hit = self.cast_ray(&ray);
			match new_hit {
				Some(new_hit) => {return self.calc_secondary_shading(&new_hit);},
				None 		=>   {return self.sample_environment(ray.dir);}
			}
		} 
		
        else 
		{
			let samples = 16;
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
						let surface = self.calc_secondary_shading(&new_hit);
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


	pub fn calc_gi(&self, hit:&Hitresult, min_samples:usize, max_samples:usize) -> Color {
		let threshold = 0.05;
		let mut rng = rand::rng();

		let mut accum = Color::black();
		let mut samples:Vec<Color> = Vec::with_capacity(max_samples);

		for i in 0..max_samples {
            let direction = Vector::cosine_sample_hemisphere(hit.normal, &mut rng);
            let ray = Ray::biased_from_direction(hit.pos, direction);
			if let Some(new_hit) = self.cast_ray(&ray) 
            {
				let dist = new_hit.pos.dist(hit.pos);
				let falloff = Self::calculate_falloff(dist);
				let color = (self.ir_read(new_hit.pos,new_hit.normal) * falloff).clamp(3.0); // clamp gi after weighting

				accum = accum + color;
				
				
				samples.push(color);
				// check if our variance is good enough to exit early
				if i >= min_samples {
					let mean = accum / (i+1) as f32;
					let v = Renderer::sample_variance(&samples, mean);
					if v < threshold { return mean;}
				}
				
				
			}
		}
		// no threshold reached
		// used all max samples
		accum  / max_samples as f32
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
		if self.settings.dof_enabled {
			fb.pixels
			.par_iter_mut()
			.enumerate()
			.for_each( |(index,color)| {
				let (x,y) = self.index_to_xy(index);
				*color = self.super_sample_defocus(x, y, 2);
			});
		}
		else {
			fb.pixels
			.par_iter_mut()
			.enumerate()
			.for_each( |(index,color)| {
				let (x,y) = self.index_to_xy(index);
				*color = self.super_sample_fixed_rate(x, y, 1);
			});
		}
		fb
    }







	fn spawn_render_progressive_thread(self) -> mpsc::Receiver<image::RgbaImage>
	{
		let (tx,rx) = mpsc::channel();
		let renderer = Arc::new(self);

		// the main renderer thread
	    thread::spawn(move || {

	    	let mut master_framebuffer = FrameBuffer::new(renderer.image_size.0, renderer.image_size.1);
			let mut iteration_counter = 1u32;
			
			loop 
			{
				master_framebuffer.write_exr();	
				println!("iteration {iteration_counter}");
	        	let samples = renderer.render_frame();
				master_framebuffer.add_buffer(&samples);
				let image = master_framebuffer.to_image();
				if tx.send(image).is_err()
				{
					return;
				}
				iteration_counter+=1;
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

