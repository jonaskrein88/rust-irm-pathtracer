use super::*;
use std::fmt;

pub struct Camera {
		pub transform: Matrix,
		pub focal_length: f32,
		pub sensor_width: f32,
		pub ratio: f32,
        pub image_width: u32,
        pub image_height: u32
	}
	impl fmt::Debug for Camera {
	    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
			let pos = self.get_pos();
	        write!(f, "Cam {} {} {}", pos.x, pos.y, pos.z)
	    }
	}
	impl Camera{
		pub fn new(image_width:u32, image_height:u32) -> Camera
		{
			let ratio = image_width as f32 / image_height as f32;
			Camera{
				transform: Matrix::identity(),
				focal_length: 50.0,
				sensor_width: 36.0, 
				ratio,
				image_width,
				image_height
			} 
		}

		pub fn calc_ray_direction(&self, pixel_x: f32, pixel_y: f32) -> Vector {
			let half_w = self.sensor_width * 0.5;
			let half_h = (self.sensor_width / self.ratio) * 0.5;

			let s = (pixel_x / self.image_width as f32) - 0.5;
			let t = 0.5 - (pixel_y / self.image_height as f32); 

			let local_pixel_pos = Vector::new(
				s * 2.0 * half_w,
				t * 2.0 * half_h,
				-self.focal_length // at the focal distance down -z
			);
			self.transform.multiply_vector(local_pixel_pos)
		}


		pub fn from_blender_transform(&mut self, values:&[[f32;4];4])
		{
			let m = 
			[
				[ values[0][0],  values[0][1],  values[0][2],  values[0][3]],
				[ values[2][0],  values[2][1],  values[2][2],  values[2][3]],
				[-values[1][0], -values[1][1], -values[1][2], -values[1][3]],
				[ values[3][0],  values[3][1],  values[3][2],  values[3][3]],
			];
			self.transform.fields.copy_from_slice(&m); 
		}


		pub fn set_lens(&mut self, focal: f32, sensor_width: f32) {
			self.focal_length = focal;
			self.sensor_width = sensor_width;
		}
		pub fn set_focal_length(&mut self, f:f32) {
			self.focal_length = f;
		}
		
		pub fn get_pos(&self) -> Vector {
			self.transform.get_translation()
		}
		pub fn walk(&mut self, distance: f32) {
			// In DCC conventions, looking forward means moving along the negative Z-axis.
			// If your engine maps forward to +Z, change this to (0.0, 0.0, distance)
			let translation_vector = Vector::new(0.0, 0.0, -distance);
			let translation_matrix = Matrix::from_translation(translation_vector);
			self.transform = self.transform.multiply(&translation_matrix);
		}
		
		
		pub fn pan_vertical(&mut self, distance: f32) {
			let translation_vector = Vector::new(0.0, distance, 0.0);
			let translation_matrix = Matrix::from_translation(translation_vector);
			self.transform = self.transform.multiply(&translation_matrix);
		}

		pub fn rotate_y(&mut self, angle_radians: f32) {
			let rotation_matrix = Matrix::from_y_rotation(angle_radians);
			self.transform = self.transform.multiply(&rotation_matrix);
		}

		pub fn rotate_x(&mut self, angle_radians: f32) {
			let rotation_matrix = Matrix::from_x_rotation(angle_radians);
			self.transform = self.transform.multiply(&rotation_matrix);
		}

		/*
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
		pub fn rotate_y(&mut self, x: f32){
			let mult = 0.01;
			self.dir.rotate_y(-x * mult);
			self.up.rotate_y(-x * mult);
			self.dir = self.dir.norm();
			self.up = self.up.norm();
		}

        // return the pixel as a [-0.5, 0.5}
        // pixel input is float to allow subpixel rendering
		pub fn pixel_to_st_coord(&self, x:f32, y:f32) -> (f32,f32) {
			let s = (x / self.image_width as f32) - 0.5;
            let t = (y / self.image_height as f32) - 0.5;
			(s * self.ratio, t)
		}

		pub fn calc_dot_pos(&self, u:f32, v:f32) -> Vector {
            let (x,y) = self.pixel_to_st_coord(u,v);
			let w = self.sensor;
			let h = self.sensor;
			let mut p = self.pos + self.dir * self.focal;
			let right = self.up.cross(self.dir);
			p = p - self.up * y * w;  	// - was +
			p + right * x * h			// - was +
		}
		*/
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
			self.origin += self.dir * bias;
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
