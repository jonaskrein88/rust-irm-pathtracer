use super::*;
use std::fmt;

pub struct Camera {
		pub pos: Vector,
		pub up: Vector,
		pub dir: Vector,
		pub focal: f32,
		pub sensor: f32,
		pub ratio: f32,
	}
	impl fmt::Debug for Camera {
	    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	        write!(f, "Cam {} {} {}", self.pos.x, self.pos.y, self.pos.z)
	    }
	}
	impl Camera{
		pub fn new() -> Camera {
			let pos = Vector::empty();
			let up = Vector::new(0.,1.,0.);
			let dir = Vector::new(0.,0.,-1.);
			let focal = 80f32;
			Camera{pos:pos, up:up, dir:dir, focal:focal,sensor:0.36, ratio:1.0} 
		}

		pub fn calc_ray_direction(&self, u:f32, v:f32) -> Vector {
			let p = self.calc_dot_pos(u,v);
			(p - self.pos).norm()
		}


		fn translate(&mut self, p:Vector) {
			self.pos = self.pos + p;
			println!("cam: {:?}",self.pos );
		}
		fn transform(&mut self, m:Matrix){ 
			let pos = self.pos;
			self.translate(-pos);
			m.apply(&mut self.pos);
			m.apply(&mut self.dir);
			m.apply(&mut self.up);
			self.translate(pos);
		}
		pub fn walk(&mut self, v:f32) {
			let mult = 10.1;
			let f = self.dir.norm();
			self.translate(f * v * mult);
		}
		pub fn dolly(&mut self, x:f32) {
			let mult = 10.1;
			let right = self.dir.norm().cross(self.up.norm()).norm() * x * mult;
			self.translate(right);
		}
		pub fn rotate(&mut self, x: f32){
			let mult = 0.01;
			self.dir.rotate_y(-x * mult);
			self.up.rotate_y(-x * mult);
			self.dir = self.dir.norm();
			self.up = self.up.norm();
		}
		pub fn calc_dot_pos(&self, u:f32, v:f32) -> Vector {
			let w = self.sensor;
			let h = self.sensor;
			let x = u - 0.5;
			let y = v - 0.5;
			let mut p = self.pos + self.dir * self.focal;
			let right = self.up.cross(self.dir);
			p = p - self.up * y * w;  	// - was +
			p + right * x * h			// - was +
		}
	}






	pub struct Ray {
		pub origin: Vector,
		pub dir:Vector,
	}

	impl Ray {
		pub fn from_direction(o: Vector, d: Vector) -> Ray { 
			Ray {origin:o, dir:d.norm()}
		}
		pub fn from_target(o: Vector, t: Vector) -> Ray {
			let dir = (t - o).norm();
			Ray::from_direction(o, dir)
		}
		pub fn biased_from_target(origin:Vector, target:Vector) -> Ray {
			let mut r = Ray::from_target(origin, target);
			r.make_bias();
			r
		}
		fn make_bias(&mut self) {
			let bias = 0.0001;
			self.origin = self.origin + self.dir * bias;
			self.dir = self.dir + self.dir * bias;
		}
		pub fn biased_from_direction(o: Vector, d: Vector) -> Ray { 
			let mut r = Ray::from_direction(o,d);
			r.make_bias();
			r
		}
		pub fn get_pos(&self, dist: f32) -> Vector {
			self.origin + self.dir * dist
		}
	}
