
use super::*;

extern crate kdtree;
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;




// color thresholds
// Default Threshold (0.12 - 0.15)
// Lower values (0.05) = higher quality, slower renders.
// Higher values (0.25) = faster renders, risk of splotches.

// Dot Threshold
// 0.966 - 15 degrees deviation (High Quality)
// 0.939 - 20 degrees deviation (Good Balance)
// 0.866 - 30 degrees deviation (Faster / Coarser)

pub struct IR_Map {
	samples: KdTree<f32, (Color,Vector), [f32;3]>,
	max_angle: f32,
	max_dist: f32,
}
impl IR_Map 
{
	pub fn new() -> Self {
		IR_Map{
			samples:KdTree::new(3),
			max_angle: 0.01,
			max_dist: 0.5,
		}
	}
	pub fn near_points(&self, p:Vector, num:usize) -> Vec<(f32, &(Color,Vector))> {
		self.samples.nearest(&p.to_array(), num, &squared_euclidean).unwrap()
	}
	pub fn add(&mut self, position:Vector, color:Color, normal:Vector) {
		self.samples.add(position.to_array(), (color,normal)).unwrap();
	} 

	// used during creation
	// check if we have a sample that is close enough
	// and if the normal is good enough so we that we can use it
	// only used for debugging
	
	pub fn get_closest_sample(&self, hit: &Hitresult) -> Option<Color> {
		let near = self.near_points(hit.pos, 8);
		for (d, &(c, n)) in near {
			if d > self.max_dist * self.max_dist {
				continue;
			}
			if n.angle_between(hit.normal) > self.max_angle {
				continue;
			}
			return Some(c);
		}
		None
	}
	
}










// kd Tree that stores Hitresults
// we use this structure to check where we need additional primary samples to refine IR

pub struct IR_Hit_Sample<'a>{
	hit:Hitresult<'a>,
	color:Color,
}

pub struct IR_Hit_Cloud<'a> {
	samples: KdTree<f32, IR_Hit_Sample<'a>, [f32;3]>,
}

impl<'a> IR_Hit_Cloud<'a> 
{
	pub fn new() -> Self {
		Self{samples:KdTree::new(3)}
	}
	pub fn near_points(&self, p:Vector, num:usize) -> Vec<(f32, &IR_Hit_Sample)> {
		self.samples.nearest(&p.to_array(), num, &squared_euclidean).unwrap()
	}
	pub fn add(&mut self, sample:IR_Hit_Sample<'a>) {
		self.samples.add(sample.hit.pos.to_array(), sample).unwrap();
	}
	pub fn compare_colors(col_a: Color, col_b: Color) -> f32 {
		// Default Threshold (0.12 - 0.15)
		// Lower values (0.05) = higher quality, slower renders.
		// Higher values (0.25) = faster renders, risk of splotches.
		let color_threshold = 0.15_f32;
		let epsilon = 1e-4_f32;

		let y_a = 0.2126 * col_a.r + 0.7152 * col_a.g + 0.0722 * col_a.b;
		let y_b = 0.2126 * col_b.r + 0.7152 * col_b.g + 0.0722 * col_b.b;
		let contrast = (y_a - y_b).abs() / (0.5 * (y_a + y_b) + epsilon);
		contrast
	}
	pub fn has_good_sample(&self, hit: &Hitresult, sampleColor:Color, max_dist:f32, max_angle:f32) -> bool {
		let near = self.near_points(hit.pos, 8);
		for (d, closest_sample) in near {
			if d > max_dist*max_dist {
				continue;
			}
			if closest_sample.hit.normal.angle_between(hit.normal).abs() > max_angle {
				continue;
			}
			if closest_sample.hit.normal.angle_between(hit.normal).abs() > max_angle {
				continue;
			}
			if Self::compare_colors(closest_sample.color, sampleColor) > 0.12{
				continue;
			}
			return true;
		}
		false
	}
}







// todo move into ir class

impl Renderer
{

	// for irradiance map precalculations
	pub fn cast_random_eye_ray(&self, rng: &mut impl Rng) -> Option<Hitresult> {
		let x = rng.random::<f32>() * self.image_size.0 as f32;
		let y = rng.random::<f32>() * self.image_size.1 as f32;
		let ray = self.ray_from_pixel(x,y);
		self.cast_ray(&ray)
	}
	// halton variant
	pub fn _cast_random_eye_ray(&self, i:i32) -> Option<Hitresult> {
		let u = Vec2::halton_sample(i);
		let x = u.x * self.image_size.0 as f32;
		let y = u.y * self.image_size.1 as f32;
		let ray = self.ray_from_pixel(x,y);
		self.cast_ray(&ray)
	}

	pub fn create_ir(&mut self) {
		let samples = self.settings.irradiance_map_primary_samples;
		let secondary_samples = 8;
		let max_depth = 2;

		let camera_pos = self.camera.get_pos();
		/*
		// Collect all the sample positions
		// samples are an iterator over Hitresults
		let gathered_hits = (0..samples)
			.into_par_iter()
			.flat_map_iter(|i| {
				// collect into Vec<Vec<Hitresult>>
				let mut path = Vec::new();
				let mut rng = SmallRng::from_rng(&mut rand::rng()); 
				
				if let Some(mut hit) = self.cast_random_eye_ray(i as i32) {
					path.push(hit);
					
					for _ in 0..(max_depth - 1) {
						let direction = Vector::cosine_sample_hemisphere(hit.normal, &mut rng);
						let ray = Ray::biased_from_direction(hit.pos,hit.normal, direction);
						
						if let Some(next_hit) = self.cast_ray(&ray) {
							path.push(hit);
							hit = next_hit;
						} else {
							break;
						}
					}
				}
				path
			}).collect::<Vec<Hitresult>>();
		*/
		

		// Finding all the sample positions first
		let mut rng = SmallRng::from_rng(&mut rand::rng()); 
		let mut sample_cloud = IR_Hit_Cloud::new();
		let mut sample_vec: Vec<Hitresult> = Vec::new();    // we need an additional flat vector since i have found no quick way to get our samples from kdtree


		// primary samples
		// firing a ton of eye samples and checking if we need to keep them or if the area has a good enough coverage already
		for _ in (0..10)
		{
			//let r = rng.random::<i32>();
			let primary_hits = (0..samples)
				.into_par_iter()
				.filter_map(|i| {
					// collect into Vec<Vec<Hitresult>>
					let mut rng = SmallRng::from_rng(&mut rand::rng()); 
					
					//if let Some(mut hit) = self.cast_random_eye_ray(r + i as i32)
					if let Some(mut hit) = self.cast_random_eye_ray(&mut rng)
					{
						//see if we found a sample close by that is good enough
						let color = self.approximate_direct_shading(&hit, camera_pos, 128, &mut rng);
						if sample_cloud.has_good_sample(&hit, color, self.irradiance_map.max_dist, self.irradiance_map.max_angle) {return None;}
						// we didn't. collect the hit and add it to the pile later
						else {return Some(IR_Hit_Sample{hit, color});}
					}
					return None;
				}).collect::<Vec<IR_Hit_Sample>>();

			for sample in primary_hits {
				sample_vec.push(sample.hit);
				sample_cloud.add(sample);
			}
		}

		


		// getting secondary sample positions
		let mut last_iteration = sample_vec.clone();

		// loop todo
		{
			let secondary_hits = last_iteration
				.into_par_iter()
				.map(|sample|
				{
					let mut rng = SmallRng::from_rng(&mut rand::rng()); 
					let mut new_hits = Vec::new();
					for _ in 0..secondary_samples
					{
						let direction = Vector::cosine_sample_hemisphere(sample.normal, &mut rng);
						let ray = Ray::biased_from_direction(sample.pos, sample.normal, direction);

						// we have a secondary hit
						if let Some(hit) = self.cast_ray(&ray)
						{
							// we found a sample close by that is good enough
							let color = self.approximate_direct_shading(&hit, sample.pos, 32, &mut rng);
							if sample_cloud.has_good_sample(&hit, color, self.irradiance_map.max_dist, self.irradiance_map.max_angle) {continue;}
							// we didn't. collect the hit and add it to the pile later
							else {new_hits.push(hit);}
						}
					}
					new_hits
					
				}).flatten().collect::<Vec<Hitresult>>();

			for hit in secondary_hits.clone() {
				//sample_cloud.add(hit);
				sample_vec.push(hit);
			}
			
			//last_iteration = secondary_hits
		}












		
		// the first iteration with only direct light
		// collects into (Position,Normal,Color)
		let hits = sample_vec
			.clone()
			.into_par_iter()
			.map(|sample|
			{
				let mut rng = SmallRng::from_rng(&mut rand::rng()); 
				let mut accum = Color::black();

				let direct = self.approximate_direct_shading(&sample, camera_pos, 128, &mut rng);
				/*
				for _ in 0..secondary_samples
				{
					let direction = Vector::cosine_sample_hemisphere(sample.normal, &mut rng);
					let ray = Ray::biased_from_direction(sample.pos, sample.normal, direction);
					// we have a secondary hit
					if let Some(hit) = self.cast_ray(&ray)
					{
						let surface = self.approximate_direct_shading(&hit, sample.pos, &mut rng);
						accum += surface;
					}
				}
				let out = direct + accum / secondary_samples as f32;
				*/
				let out = direct;
				(sample.pos, sample.normal, out)
				
			}).collect::<Vec<(Vector,Vector,Color)>>();

		
		let mut ir = IR_Map::new();
		for hit in hits {
			ir.add(hit.0, hit.2, hit.1);
		}



		// now we are starting a recursive loop that will sample the ir in addition to direct light
		// and overwrite the ir

		
		
		for _ in (0..3)
		{
			let hits = sample_vec
			.clone()
			.into_par_iter()
			.map(|sample|
			{
				let mut rng = SmallRng::from_rng(&mut rand::rng()); 
				let mut accum = Color::black();

				let direct = self.approximate_direct_shading(&sample, camera_pos, 64, &mut rng);
				let material = self.get_material(&sample);
				let albedo = material.get_albedo(&self, sample.uv); 

				for _ in 0..secondary_samples
				{
					let direction = Vector::cosine_sample_hemisphere(sample.normal, &mut rng);
					let ray = Ray::biased_from_direction(sample.pos, sample.normal, direction);
					// we have a secondary hit
					if let Some(hit) = self.cast_ray(&ray)
					{
						let other_direct = self.approximate_direct_shading(&hit, sample.pos, 128, &mut rng);
						let other_gi = self.ir_read(&self.irradiance_map, hit.pos, hit.normal);
						accum += other_direct + other_gi;
					}
				}
				let out = direct + accum / secondary_samples as f32 * albedo;
				(sample.pos, sample.normal, out)
				
			}).collect::<Vec<(Vector,Vector,Color)>>();

			
			let mut new_ir = IR_Map::new();
			for hit in &hits {
				new_ir.add(hit.0, hit.2, hit.1);
			}
			ir = new_ir;
		}
		

		self.irradiance_map = ir;
	}
	




	pub fn ir_read(&self, ir:&IR_Map, pos: Vector, normal: Vector) -> Color {
		let near_points = ir.near_points(pos, 20);

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

			// gaussian spatial falloff
			let distance_weight = (-0.5 * (d / sigma).powi(2)).exp();

			let normal_dot = normal.dot(sample_normal);
			if normal_dot <= 1e-3_f32 {
				continue; 
			}
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
	pub fn ir_render_samples(&self, hit:&Hitresult) -> Color 
	{
		if let Some(color) = self.irradiance_map.get_closest_sample(hit) {return color}
		Color::black()
	}

}	