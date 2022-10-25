pub mod vectormath;
pub mod camera;
pub mod geometry;
pub mod material;
pub mod color;

extern crate kdtree;
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;

use vectormath::*;
use color::*;
use camera::*;
use geometry::*;
use material::*;
use std::mem;


const PI: f32 		= 3.14159265359; 
const TWO_PI: f32 	= 6.2831832; 



pub struct Framebuffer {
    pub pixels: Vec<Vec<Color>>,
    pub width: usize,
    pub height: usize
}

impl Framebuffer{
    pub fn new(x: usize, y:usize) -> Framebuffer 
    {
        let mut buff:Vec<Vec<Color>> = Vec::with_capacity(x);
        for _ in 0..x {
            let mut col: Vec<Color> = Vec::with_capacity(y);
            for _ in 0..y {col.push(Color::black());}
            buff.push(col);
        }
        Framebuffer{pixels:buff,width:x, height:y}
    }
    pub fn clear(&mut self) {
        for i in 0..self.pixels.len() {
            for j in 0..self.pixels[i].len() {
                self.pixels[i][j] = Color::black();
            }
        }
    }
    pub fn get(&self,x:usize, y:usize) -> Color {
        self.pixels[x][y]
    }
    pub fn set(&mut self, x:usize, y:usize, value:Color) {
        self.pixels[x][y] = value;
    }
}









pub struct Settings {
		environment:Color,
		shadows: bool,
		render_alpha: bool,
		render_normals: bool,
		render_uv: bool,
		gi_method_primary: GiMethod,
		gi_method_secondary: GiMethod,
		gi_primary_ray_count: usize,
		gi_depth: i32,
		gi_on: bool,
		reflections_on: bool,
		surface_on :bool,
	}
	impl Settings {
		fn new() -> Settings {
			Settings { 
				shadows:true, 
				environment: Color::black(),
				render_alpha: false,
				render_normals: false,
				render_uv:false,
				gi_method_primary: GiMethod::IrradianceMap,
				gi_method_secondary: GiMethod::IrradianceMap,
				gi_primary_ray_count: 10,
				gi_depth: 3,
				gi_on: true,
				reflections_on: true,
				surface_on :true,

			}
		}
	}






	pub enum GiMethod {
		BruteForce,
		IrradianceMap,
		LightCache,
	}






pub struct Lightcache {
    samples: KdTree<f64, Color, [f64;3]>,
}
impl Lightcache {
    fn new() -> Lightcache {
        Lightcache{samples:KdTree::new(3)}
    }
    fn add(&mut self, p:Vector, c:Color) {
        self.samples.add(p.to_array(), c).unwrap();
    }
    fn near_points(&self, p:Vector, num:usize) -> Vec<(f64, &Color)> {
        self.samples.nearest(&p.to_array(), num, &squared_euclidean).unwrap()
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
		image_size: (usize,usize),
		frame:Framebuffer,
		settings:Settings,
		light_cache:Lightcache,
		irradiance_map: IRClass,
	}
	impl Renderer {
		pub fn new(x: usize, y:usize) -> Renderer { 
			let nodes: Vec<Geo> = Vec::new(); 
			let lights: Vec<Light> = Vec::new(); 
			let mut cam = Camera::new();
			cam.ratio = x as f32 / y as f32;
			let mut r = Renderer {
				camera: cam, 
				nodes: nodes, 
				lights:lights, 
				image_size:(x,y),
				frame:Framebuffer::new(x,y),
				settings: Settings::new(),
				light_cache:Lightcache::new(),
				irradiance_map: IRClass::new(),
			};
			r.init();
			r
		}
		pub fn add(&mut self, mut g:Geo) { 
			g.prepass();
			self.nodes.push(g);
		}
		pub fn clear_buffer(&mut self) { self.frame.clear();}

		pub fn init(&mut self) {

			let mut s2 = Geo::new();
			s2.pos.z = 0.0;
			s2.pos.x = 10.;
			s2.pos.y = -5.;
			s2.shape = Shape::Sphere(12.);
			s2.material.spec = BRDF::Phong;
			//s2.material.refraction = true;
			s2.material.reflection = true;
			//s2.material.diffuse = BRDF::None;
			self.add(s2);

			let mut s2 = Geo::new();
			s2.pos.z = -15.;
			s2.pos.x = 2.;
			s2.pos.y = -5.;
			s2.shape = Shape::Sphere(8.);
			s2.material.spec = BRDF::Phong;
			s2.material.reflection = true;
			//s2.material.diffuse = BRDF::None;
			//self.add(s2);

			let mut s = Geo::create_sphere(Vector::new(-2.,-8.,-20.));
			s.material.color = Color::red();
			s.set_size(3.);
			//self.add(s);



			let mut s1 = Geo::create_sphere(Vector::new(-7.,-7.,-20.));
			//s1.material.reflection = true;
			s1.material.ior = 0.1;
			s1.material.glossiness = 0.7;
			//s1.material.color = Color::black();
			//s1.material = Material::new_light_material();
			s1.set_size(4.);
			//self.add(s1);

			//let nmap = Texture::open("C:/Users/Jo/Desktop/normal.jpg");
			//let tex = Texture::open("C:/Users/Jo/Desktop/download.png");
			let mut mesh = open("C:/Users/j.krein/Desktop/box.obj");
			//let mut mesh = open("C:/Users/Jo/Desktop/tea_pot.obj");
			//let mut mesh = open("C:/Users/Jo/Desktop/box.obj");
			mesh.scale(0.1);
			//mesh.translate(Vector::new(0.,0.,-5.));
			let mut mg = Geo::create_mesh(mesh);
			//mg.material.texmap_diffuse = Some(tex);
			//mg.material.texmap_normal = Some(nmap);
			//mg.material.reflection = true;
			self.add(mg);


			//let tex2 = Texture::open("C:/Users/Jo/Desktop/uv.jpg");
			let mut floor = Polymesh::new();
			floor.add_plane();
			floor.transform_uvs(5.,5.);
			let g = Geo::create_mesh(floor);
			//g.material.texmap_diffuse = Some(tex2);
			self.add(g);

			let mut wall = Polymesh::new();
			wall.add_plane();
			wall.transform_uvs(5.,5.);
			wall.transform(Matrix::rotate_x(90.));
			wall.translate(Vector::new(0.,0.,-30.));
			let w = Geo::create_mesh(wall);
			//g.material.texmap_diffuse = Some(tex2);
			self.add(w);


			let mut l = Light::new();
			l.pos.y = 5.;
			l.pos.x = 5.;
			l.color = Color::new(0.5,0.5,1.0);
			self.lights.push(l);

			let mut l2 = Light::new();
			l2.pos.y = 5.;
			l2.pos.x = -5.;
			l2.color = Color::new(1.,1.,0.5);
			self.lights.push(l2);


            println!("starting IR generation");
			//self.create_ir();
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
		pub fn create_light_cache(&mut self) {
			self.light_cache = self.light_cache_primary();
		}
		pub fn light_cache_primary(&self) -> Lightcache {
			pub fn light_cache_secondary(r: &Renderer, hit:&Hitresult, cache:&mut Lightcache, level:usize) -> Color 
			{
				let mut out = Color::black();
				let samples = 20;
				for _ in 0..samples {
					let dir = hit.normal.random_hemispherical();
					let hit_opt = {
						let ray = Ray::biased_from_direction(hit.pos, dir);
						r.cast_ray(&ray)
					}; 
					if let Some(new_hit) = hit_opt {
						let mut surface_color = r.calc_color(&new_hit, hit.pos);
						if level > 0 {
							let gi = light_cache_secondary(r, &new_hit, cache, level - 1);
							surface_color = surface_color + gi;
						}
						if level > 0 { cache.add(new_hit.pos, surface_color);}
						//let dist = hit.pos.dist(new_hit.pos);
						//let falloff = 1. / (dist * 0.8).max(1.);
						out = out + surface_color;
						
					}
				}
				out / samples as f32
			}
			let mut cache = Lightcache::new();
			for _i in 0..1000 {
					if let Some(hit) = self.cast_random_ray() {
					let surface_color = self.calc_color(&hit, self.camera.pos);
					let gi = light_cache_secondary(self,&hit, &mut cache,  0);				
					cache.add(hit.pos, surface_color + gi);	
					}
			}
			cache
		}
		pub fn light_cache_read(&self, pos:Vector) -> Color {
			let near_points = self.light_cache.near_points(pos, 1);
			return *near_points[0].1;/*
			let mut out = Color::black();
			let mut accum = 0.0;
			for s in near_points {
				let (dist, &col) = s;
				let denom = 1.0/dist.max(0.0001) as f32;
				out = out + col * denom;
				accum += denom;
			}
			out / accum as f32
			*/
		}
		pub fn create_ir(&mut self) {
			let samples = 1500;
			let mut i = 0;
			let mut ir = IRClass::new();
			while i < samples{
				if let Some(hit) = self.cast_random_ray() {
					match ir.check_sample(&hit) {
						Some(_) => (),
						None =>  {
							let mut out = self.approximate_color(&hit, self.camera.pos);
							if self.reads_gi(&hit.node.material) {out += self.ir_secondaray(&hit, &mut ir, 2);}
							ir.add(hit.pos, out, hit.normal);
							i += 1;
						}
					}
				}
			}
			self.irradiance_map = ir;
		}
		pub fn ir_secondaray(&self, hit:&Hitresult, ir:&mut IRClass, level:usize) -> Color {
			let mut out = Color::black();
			let samples = 5;
			for _i in 0..samples {
				let dir = hit.normal.random_hemispherical();
				let ray = Ray::biased_from_direction(hit.pos, dir);
				if let Some(new_hit) = self.cast_ray(&ray) {
					let c = match ir.check_sample(&new_hit) {
						Some(col) => col,
						None =>  {
							let mut surface = self.approximate_color(&new_hit, self.camera.pos);
							if level > 0 && self.reads_gi(&new_hit.node.material) {
								let gi = self.ir_secondaray(&new_hit, ir, level-1) * hit.node.material.color;
								surface = surface + gi;
							}
							if level > 0 { ir.add(new_hit.pos, surface, new_hit.normal);}
							surface
						}
					};
					out = out + c
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
		pub fn ir_render_samples(&self, hit:&Hitresult) -> Color {
			let v = self.irradiance_map.near_points(hit.pos, 1);
			if let Some(&(p,_)) = v.get(0) {
				if p < 0.01 { return Color::white();}
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
					if level > 2 {return r.approximate_color(&new_hit, hit.pos);}
					else if new_hit.node.material.refraction {
						return calc_refraction_secondary(r, &new_hit, hit.pos, level+1)
					}
					else {return r.approximate_color(&new_hit, hit.pos) + r.calc_gi(&new_hit);}
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
		pub fn pixel_to_float(&self, s:f32, t:f32) -> (f32,f32) {
			let (w,h) = self.image_size;
            let ratio = w as f32 / h as f32;
			let x = s / w as f32 * ratio;
            let y = t / h as f32 ;
			(x,y)
		}
		pub fn ray_from_pixel(&self, x: f32, y: f32) -> Ray{
			let (s,t) = self.pixel_to_float(x,y);
			let dir = self.camera.calc_ray_direction(s,t);
			Ray::from_direction(self.camera.pos, dir)
		}
		pub fn super_sample_fixed_rate(&self, x: f32, y:f32) -> Color {
			let samples = 4;
			let mut out = Color::black();
			for _ in 0..samples {
				let mut ray = self.ray_from_pixel(x + rand::random::<f32>(),y + rand::random::<f32>());
				let c = self.cast_primary_ray(&ray);
				out += c;
			}
			out / (samples * samples) as f32
		}

		pub fn super_sample_bokeh(&self, x: f32, y:f32) -> Color {
			let samples = 20;
			let focal_dist = 10.0;
			let mut out = Color::black();
			let r = 0.2;

			let up = self.camera.up;
			let right = self.camera.dir.cross(up);

			let old_dir = {
				let (s,t) = self.pixel_to_float(x,y);
				self.camera.calc_ray_direction(s,t)
			};
			let focal_point = self.camera.pos + old_dir * focal_dist;


			for i in 0..samples {
				let u = (i as f32 * TWO_PI / samples as f32 ).cos() * r;
				let v = (i as f32 * TWO_PI / samples as f32 ).sin() * r;

				let origin = self.camera.pos + right * u + up * v;

				let dir = (focal_point - origin).norm();
				let ray = Ray::from_direction(origin, dir);

				out += self.cast_primary_ray(&ray);
			}
			out / samples as f32
		}

		pub fn measure_variance(&self, x: f32, y:f32) -> Color {
			let max = 8;
			let mut accum = Color::black();
			let mut samples:Vec<Color> = Vec::new();
			for _ in 0..max {
				let xr = rand::random::<f32>();
				let yr = rand::random::<f32>();
				let ray = self.ray_from_pixel(x + xr, y + yr);
				if let Some(hit) = self.cast_ray(&ray) {
					let color = self.calc_color(&hit, self.camera.pos);
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
		pub fn approximate_color(&self, hit:&Hitresult, origin:Vector) -> Color {
			if hit.node.material.reflection {
				let fresnel = self.calc_fresnel(hit.pos, self.camera.pos,hit.normal, hit.node.material.ior);
				let hit_opt = self.cast_reflection_ray(origin, hit);
				let reflect = match hit_opt {
					Some(new_hit) => {self.calc_color(&new_hit, hit.pos)},
					None 		=> {self.settings.environment}
				};
				let surface = self.calc_color(&hit,self.camera.pos) * (1. - fresnel);
				return reflect + surface
			}
			self.calc_color(hit, origin)
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
								//return self.ir_render_samples(&hit);
								//return hit.tangent.normal_to_color();
								//return hit.normal.normal_to_color();
								//return hit.normal.random_hemispherical().normal_to_color();
								//return random_hemispherical(hit.normal).normal_to_color();
								//return self.light_cache_read(hit.pos);
								let mut surface = self.calc_color(&hit,self.camera.pos);
								if hit.node.material.refraction {surface = surface + self.calc_refraction(&hit, self.camera.pos)}
											
								let mut out = if hit.node.material.reflection {
									let fresnel = self.calc_fresnel(hit.pos, self.camera.pos,hit.normal, hit.node.material.ior);
									let reflect = self.calc_reflection(&hit,self.camera.pos,0) * fresnel;
									reflect + surface * (1. - fresnel)
								} else{
									surface
								}; 
								//out = out + self.calc_gi(&hit) * hit.node.material.color;
                                out = out/ 2.0;
								out

							}
			}
		}

        // returns the brdf color
        // no reflection, refraction, gi
		pub fn calc_color(&self, hit:&Hitresult, origin:Vector) -> Color {
			if let BRDF::Constant = hit.node.material.diffuse {return hit.node.material.color}
			let mut out = Color::black();
				for light in &self.lights
				{
					if self.settings.shadows && self.check_shadow(hit.pos, &light) {continue;}
					let diffuse = {
						if hit.node.material.color == {Color::black()} {Color::black()}
						else {hit.node.material.calc_diffuse(hit.pos, hit.normal, hit.tangent, &light, origin, hit.uv)}
					};
					let spec = { 
						if hit.node.material.reflection_color == Color::black() {Color::black()}
						else {hit.node.material.calc_specular(hit.pos, hit.normal, &light, origin)}
					};
					out = out + diffuse + spec;
				}
			return out;
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
		pub fn calc_reflection(&self, hit:&Hitresult, origin:Vector, level:usize) -> Color {
			if level > 2 {return Color::black();}
			if hit.node.material.glossiness == 1.0 {
				let new_hit = self.cast_reflection_ray(origin, hit);
				match new_hit {
					Some(new_hit) => {return self.calc_color(&new_hit, hit.pos);},
					None 		=> {return self.settings.environment;}
				}
			} else {
				let samples = 64;
				let mut out = Color::black();
				for _ in 0..samples {
					let view = hit.pos - origin;
					let reflect = Vector::calc_reflect_vector(view, hit.normal);
					let rand = reflect.random_angle(PI/5.);
					let ray = Ray::biased_from_direction(hit.pos, rand);
					match self.cast_ray(&ray) {
						None => {out = out + self.settings.environment;},
						Some(new_hit) => {
							let surface = self.calc_color(&new_hit, hit.pos);
							out = out + surface;
						},
					}
				}
				out / samples as f32
			}
		}
		pub fn cast_reflection_ray(&self, origin:Vector, hit:&Hitresult) -> Option<Hitresult> {
			let view = hit.pos - origin;
			let reflect_vec = Vector::calc_reflect_vector(view, hit.normal);
			let ray = Ray::biased_from_direction(hit.pos, reflect_vec);
			self.cast_ray(&ray)
		}
		pub fn render(&self, x: i32, y: i32) -> (u8,u8,u8) {
			let r = self.ray_from_pixel(x as f32,y as f32);
			self.cast_primary_ray(&r).to_0_255()
		}

		pub fn calc_gi(&self, hit:&Hitresult) -> Color {
			if !self.reads_gi(&hit.node.material) {return Color::black()}
			match self.settings.gi_method_primary {
				GiMethod::BruteForce 		=> self.gi_brute_force(hit, 0),
				GiMethod::IrradianceMap 	=> self.ir_read(hit.pos),
				GiMethod::LightCache 		=> self.light_cache_read(hit.pos),
			}
		}
		pub fn gi_brute_force(&self, hit:&Hitresult, level:i32) -> Color { 
			let passes = (24. / 2.0f32.powi(level)) as usize;
			let mut out_color = Color::black();
			for _ in 0..passes {
				if let Some(new_hit) = self.cast_random_hemispherical_ray(hit.pos, hit.normal) {
					if hit.pos.dist(new_hit.pos) < 50. {
						let color = match self.settings.gi_method_secondary 
						{
							GiMethod::IrradianceMap 	=> self.ir_read(new_hit.pos),
							GiMethod::BruteForce 		=> {
								let mut color = self.approximate_color(&new_hit, hit.pos);
								if level < 1 { 
									let secondary_gi = self.gi_brute_force(&new_hit, level+1);
									color += secondary_gi * hit.node.material.color;                // this was unused !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
								}
								color
							},
							GiMethod::LightCache 		=> self.light_cache_read(new_hit.pos),	
						};
						//let dist = new_hit.pos.dist(hit.pos);
						//let falloff = 1. / (dist * 0.1).max(1.);
						out_color = out_color + color;
					}
				}
			}
			out_color * hit.node.material.color / passes as f32 * 0.6
		}
		pub fn gi_brute_force2(&self, hit:&Hitresult) -> Color {
			let mut accum = Color::black();
			let mut samples:Vec<Color> = Vec::new();
			let min = 4;
			let max = 32;
			for i in 0..max {
				if let Some(new_hit) = self.cast_random_hemispherical_ray(hit.pos, hit.normal) {
					let color = self.ir_read(new_hit.pos);
					accum = accum + color;
					samples.push(color);
					if i >= min {
						let mean = accum / (i+1) as f32;
						let v = Renderer::sample_variance(&samples, mean);
						if v < 0.02 { return mean * hit.node.material.color}
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
		

	}

