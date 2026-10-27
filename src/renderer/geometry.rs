use super::*;

use std::fs::File;
use std::str::{SplitWhitespace, Lines};
use std::io::Read;
use std::time;
use std::collections::HashMap;

use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::{BHShape, BoundingHierarchy};
use bvh::bvh::Bvh;
use bvh::ray::Ray as BvhRay;
use nalgebra::{Point3, Vector3};





pub struct Light {
    pub color: Color,
    pub pos: Vector,
    pub power: f32,
}
impl Light {
    pub fn new() -> Light{Light{pos:Vector::empty(), color:Color::white(), power:0.3}}

    pub fn calc_falloff(&self, pos:Vector) -> f32 {
        let dist = (pos - self.pos).length();
        let fall = dist.powi(2) * 4. * 3.14159265359;
        3000. * self.power / fall
        //self.power
    }
}








#[derive(Debug,Copy,Clone)]
pub struct Face {
    a:usize,
    b:usize,
    c:usize,
	na:usize,
	nb:usize,
	nc:usize,
    uva:usize,
    uvb:usize,
    uvc:usize,
	//tana:usize,
	//tanb:usize,
	//tanc:usize,
	material_id:usize
}

impl Face
{
    pub fn new( 
		verts:  (usize,usize,usize),
		normals:(usize,usize,usize),
		uvs:    (usize,usize,usize)
	) -> Face {
        let face = Face{
			a:verts.0, b:verts.1, c:verts.2,
			uva:uvs.0, uvb:uvs.1, uvc:uvs.2,
			na:normals.0, nb:normals.1, nc:normals.2,
			//tana:0, tanb:0, tanc:0,
			material_id:0
		};
		face
    }
	
}



// proxy triangle for BVH
struct BvhTriangle {
    pub original_face_index: usize,
    pub aabb: Aabb<f32, 3>,
	pub node_index: usize

}
impl Bounded<f32, 3> for BvhTriangle {
    fn aabb(&self) -> Aabb<f32, 3> {
        self.aabb
    }
}
impl BHShape<f32, 3> for BvhTriangle {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }
    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}





	pub struct Points(Vector, Vector, Vector);

	pub struct Polymesh
	{
		faces:     Vec<Face>,
		vertices:  Vec<Vector>,
		fnormals:  Vec<Vector>,
		ftangents: Vec<Vector>,
		aabb:     (Vector,Vector),
		uvs:       Vec<Vec2>,
		vnormals:  Vec<Vector>,
		vtangents: Vec<Vector>,
		num_materials : usize,
		bvh :             Option<Bvh<f32,3>>,
		bvh_proxy_faces : Vec<BvhTriangle>
	}

	impl Polymesh
	{
		pub fn new() -> Polymesh {
			Polymesh { 
				faces:Vec::new(), 
				vertices:Vec::new(),
				fnormals:Vec::new(),
				ftangents: Vec::new(),
				aabb:(Vector::empty(),Vector::empty()),
				uvs:Vec::new(),
				vtangents: Vec::new(),
				vnormals:Vec::new(),
				num_materials:1,
				bvh:None,
				bvh_proxy_faces:Vec::new(),
			}
		}
		fn pre_calculate_aabb(&mut self) {
			let mut max = Vector::new(f32::MIN,f32::MIN,f32::MIN);
			let mut min = Vector::new(f32::MAX,f32::MAX,f32::MAX);
			for v in &self.vertices {
				max.x = v.x.max(max.x);
				max.y = v.y.max(max.y);
				max.z = v.z.max(max.z);
				min.x = v.x.min(min.x);
				min.y = v.y.min(min.y);
				min.z = v.z.min(min.z);
			}
			self.aabb = (min,max);
		}

		fn prepare_for_bvh(&mut self)
		{
			// pre-calculating the bounding boxes up front
			let mut bvh_triangles: Vec<BvhTriangle> = self.faces.iter().enumerate().map(|(idx, face)| {
				let p0 = self.vertices[face.a];
				let p1 = self.vertices[face.b];
				let p2 = self.vertices[face.c];

				let min = Point3::new(
					p0.x.min(p1.x).min(p2.x),
					p0.y.min(p1.y).min(p2.y),
					p0.z.min(p1.z).min(p2.z),
				);
				let max = Point3::new(
					p0.x.max(p1.x).max(p2.x),
					p0.y.max(p1.y).max(p2.y),
					p0.z.max(p1.z).max(p2.z),
				);
				BvhTriangle {
					original_face_index: idx,
					aabb: Aabb::with_bounds(min, max),
					node_index: 0 // default
				}
			}).collect();
			self.bvh = Some(Bvh::build(&mut bvh_triangles));
			self.bvh_proxy_faces = bvh_triangles;
		}



		// traverse bvh
		// get the original faces of our candidate proxies
		// pass to iterate faces
		// returns the result from iterate_faces
		fn cast_ray_with_bvh(&self, ray:&Ray) -> Option<(Vector, Vector, Vec2, Vector, usize)> 
		{
			// convert our Ray to BvHRay
			let bvh_ray = BvhRay::new(
				Point3::new(ray.origin.x, ray.origin.y, ray.origin.z), 
				Vector3::new(ray.dir.x, ray.dir.y, ray.dir.z)
			);

			
			if let Some(ref bvh) = self.bvh
			{
				// traverse the hierarchy
				// returns a Vec<&BvhTriangle> containing only the faces whose bounding boxes were hit
				let potential_hits = bvh.traverse(&bvh_ray, &self.bvh_proxy_faces);
				let mut best_dist = f32::MAX;
				let mut best_hit: Option<(Vector,&Face,f32,f32,f32)> = None;
				
				for proxy in potential_hits
				{
					let face = &self.faces[proxy.original_face_index];
					let hit = self.calculate_hit(ray, face, &self.fnormals[proxy.original_face_index]);
					match hit {
						None => continue,
						Some((pos,..)) => {
							let dist = (ray.origin - pos).length();
							if dist > 1e-4 && dist < best_dist {
								best_dist = dist;
								best_hit = hit;
							}
						}
					}
				}
				match best_hit {
					None => return None,
					Some((pos, f, u,v,w)) => {
						let normal = self.interpolate_normal(f,(u,v,w));
						let uv = self.interpolate_uv(f,(u,v,w));
						let tangent = self.interpolate_tangent(f,(u,v,w));
						//let tangent = Vector::new(1.0,0.0,0.0);
						//let uv = Vec2::new(0.0,0.0);

						return Some((pos, normal, uv, tangent, f.material_id))
					},
				}
			}
			panic!("no bvh found on mesh");
			None
		} 
		

		// this is incorrect at the moment ant needs a rework
		//////////////////////////////////////////////////////////////
	
		
		// required for vertex tangent calculation
		fn faces_from_point(&self, v:usize) -> Vec<usize> {
			let mut out:Vec<usize> = Vec::new();
			for i in 0..self.faces.len() {
				let face = &self.faces[i];
				if face.a == v || face.b == v || face.c == v { out.push(i);}
			}
			out
		}

		fn calculate_face_tangents(&self, f: &Face) -> Vector {
			
			let Points(v0, v1, v2) = self.get_positions(f);
			let uv0 = self.uvs[f.uva];
			let uv1 = self.uvs[f.uvb];
			let uv2 = self.uvs[f.uvc];

			let delta1 = v1 - v0;
			let delta2 = v2 - v0;
			let delta_uv_1 = uv1 - uv0;
			let delta_uv_2 = uv2 - uv0;

			let det = delta_uv_1.x * delta_uv_2.y - delta_uv_1.y * delta_uv_2.x;

			if det.abs() < 1e-6 {
				return Vector::zero();
			}

			let r = 1.0 / det;
			(delta1 * delta_uv_2.y - delta2 * delta_uv_1.y) * r
		}

		/*
		fn _pre_calculate_vtangents(&mut self)
		{
			println!("starting vtan calculation");
			if self.ftangents.len() == 0 {return ();}
			for i in 0..self.vertices.len(){
				let faces = self.faces_from_point(i);
				let mut tangent = Vector::empty();
				for i in 0..faces.len() {
					let ftangent = self.ftangents[faces[i]];
					tangent = tangent + ftangent;
				}
				self.vtangents.push(tangent.norm());
			}
			println!("done vtan calculation");
		}
		*/

		fn pre_calculate_vtangents(&mut self) {
			if self.ftangents.is_empty() || self.faces.is_empty() { return; }

			let mut accumulated = vec![Vector::empty(); self.vertices.len()];

			for (face_idx, face) in self.faces.iter().enumerate() {
				let ftangent = self.ftangents[face_idx];
				accumulated[face.a] = accumulated[face.a] + ftangent;
				accumulated[face.b] = accumulated[face.b] + ftangent;
				accumulated[face.c] = accumulated[face.c] + ftangent;
			}
			self.vtangents = accumulated
				.into_iter()
				.map(|v| v.norm())
				.collect();
		}

		//////////////////////////////////////////////////////////////


		// run before rendering
		// after all transformations
		pub fn prepass(&mut self) {
			self.pre_calculate_aabb();
			self.pre_calculate_fnormals_and_ftangents();
			self.pre_calculate_vtangents();
			self.prepare_for_bvh();
		}

		pub fn transform(mut self, m:&Matrix) -> Self{
			for v in &mut self.vertices {
				*v = m.multiply_point(*v);
			}
			self
		}

		pub fn transform_normals(mut self, m:&Matrix) -> Self{
			for v in &mut self.vnormals {
				*v = m.multiply_vector(*v);
			}
			for v in &mut self.fnormals {
				*v = m.multiply_vector(*v);
			}
			self
		}

		pub fn translate(mut self, t:Vector) -> Self{
			for v in &mut self.vertices {
				*v = v.add(t);
			}
			self
		}
		pub fn translate_y(mut self, t:f32) -> Self{
			self.translate(Vector::new(0.0,t,0.0))
		}
		pub fn scale(mut self, f:f32) -> Self{
			let m = Matrix::from_scale(f);
			self.transform(&m)
		}
		pub fn rotate_y(mut self, f:f32) -> Self{
			let m = Matrix::from_y_rotation(f);
			self.transform(&m).transform_normals(&m)
		}
		
		
		fn get_positions(&self, f:&Face) -> Points {
			let a = self.get_position(f.a);
			let b = self.get_position(f.b);
			let c = self.get_position(f.c);
			Points(a,b,c)
		}
		fn get_position(&self, i:usize) -> Vector {
			let vec_opt = self.vertices.get(i);
			match vec_opt {
				Some(v) => *v,
				None => {panic!("Face index out of range");},
			}
		}
		
		// legacy; for non BVH
		// Option(pos, normal, uv, tangent, mtl_id)
		fn iterate_faces(&self, ray:&Ray) -> Option<(Vector, Vector, Vec2, Vector, usize)>
		{
			if !self.calculate_aabb_intersection(ray) { return None;}
			let mut best_dist = f32::MAX;
			let mut best_hit: Option<(Vector,&Face,f32,f32,f32)> = None;
			for i in 0..self.faces.len() {
				let hit = self.calculate_hit(ray, &self.faces[i], &self.fnormals[i]);
				match hit {
					None => continue,
					Some((pos,..)) => {
						let dist = (ray.origin - pos).length();
						if dist > 1e-4 && dist < best_dist {
							best_dist = dist;
							best_hit = hit;
						}
					}
				}
			}
			match best_hit {
				None => None,
				Some((pos, f, u,v,w)) => {
					let normal = self.interpolate_normal(f,(u,v,w));
					let uv = self.interpolate_uv(f,(u,v,w));
					//let tangent = if self.has_uvs {self.interpolate_tangent(f,(u,v,w))} else {Vector::new(1.,0.,0.)};
                    let tangent = Vector::new(1.0,0.0,0.0);
                    //let uv = Vec2::new(0.0,0.0);

					Some((pos, normal, uv, tangent, f.material_id))
				},
			}
		}
		
		// returns Option ( position, Face, Barycentic)
		fn calculate_hit<'a>(&self, ray: &Ray, f:&'a Face, n:&Vector) -> Option<(Vector, &'a Face, f32,f32,f32)> 
		{
			if n.dot(ray.dir) > 0.0 { return None;} // uncomment for backface culling
			// Möller-Trumbore
			let det_epsilon = 1e-8;
			let t_epsilon   = 1e-5;

			let p = self.get_positions(f);
			//let k_epsilon = 0.001;
			let v0v1 = p.1 - p.0;
			let v0v2 = p.2 - p.0;
			let pvec = ray.dir.cross(v0v2); 
			let det = v0v1.dot(pvec);
			if det < det_epsilon {return None};  // remove abs() for backface culling
			let inv_det = 1. / det;
			let tvec = ray.origin - p.0;
			let u = tvec.dot(pvec) * inv_det;
			if u < 0. || u > 1. {return None;}
			let qvec = tvec.cross(v0v1);
			let v = ray.dir.dot(qvec) * inv_det;
			if v < 0. || u + v > 1. { return None;}
			let t = v0v2.dot(qvec) * inv_det;
            if t < t_epsilon {return None;}
			let hit_pos = ray.origin + ray.dir * t;
			let w = 1. - u - v;
			Some((hit_pos, f, u,v,w))
		}
		fn pre_calculate_fnormals_and_ftangents(&mut self) {
			for f in &self.faces {
				let points = self.get_positions(f);
				let n = self.calculate_normal(&points);
				self.fnormals.push(n);
				let t = self.calculate_face_tangents(&f);
				self.ftangents.push(t);
			}
		}
		fn calculate_normal(&self, p:&Points) -> Vector {
			let e1 = p.1 - p.0;
			let e2 = p.2 - p.0;
			e1.cross(e2).norm()
		}
		fn interpolate_normal(&self, f:&Face, (u,v,w):(f32,f32,f32)) -> Vector {
			let c = self.vnormals[f.na];
			let a = self.vnormals[f.nb];
			let b = self.vnormals[f.nc];
			(a * u + b * v + c * w).norm()
		}
		fn interpolate_tangent(&self, f:&Face, (u,v,w):(f32,f32,f32)) -> Vector {
			let c = self.vtangents[f.a];
			let a = self.vtangents[f.b];
			let b = self.vtangents[f.c];
			a * u + b * v + c * w
		}
		fn interpolate_uv(&self, f:&Face, (u,v,w):(f32,f32,f32)) -> Vec2 {
			let c = self.uvs[f.uva];
			let a = self.uvs[f.uvb];
			let b = self.uvs[f.uvc];
			a * u + b * v + c * w
		}
		fn calculate_aabb_intersection(&self, ray:&Ray) -> bool {
			let mut tmin = (self.aabb.0.x - ray.origin.x) / ray.dir.x;
			let mut tmax = (self.aabb.1.x - ray.origin.x) / ray.dir.x;
			if tmin > tmax { mem::swap(&mut tmin, &mut tmax);}
			let mut tymin = (self.aabb.0.y - ray.origin.y) / ray.dir.y;
			let mut tymax = (self.aabb.1.y - ray.origin.y) / ray.dir.y;
			if tymin > tymax { mem::swap(&mut tymin, &mut tymax);}
			if tmin > tymax || tymin > tmax { return false;}
			if tymin > tmin {tmin = tymin;} 
	 		if tymax < tmax {tmax = tymax;}
	 		let mut tzmin = (self.aabb.0.z - ray.origin.z) / ray.dir.z;
			let mut tzmax = (self.aabb.1.z - ray.origin.z) / ray.dir.z;
			if tzmin > tzmax { mem::swap(&mut tzmin, &mut tzmax);}
			if tmin > tzmax || tzmin > tmax { return false;}
			//if tzmin > tmin {tmin = tzmin;} 
	    	//if tzmax < tmax {tmax = tzmax;} 
	    	return true;  
		}
	}





	pub enum Shape {
		Sphere(f32),
		Mesh(Polymesh),
		Plane(Vector, Vector),
	}


	pub struct Geo {
		pub id: u32,
		pub pos: Vector,
		pub shape: Shape,
		pub materials: Vec<usize>
	}

	impl Geo
	{
		pub fn new() -> Geo
		{ 
			Geo {
				id:0,
				pos:Vector::empty(),
				shape:Shape::Sphere(1.0),
				materials:Vec::new()
			}
		}

		pub fn create_sphere(pos: Vector, size:f32) -> Self {
			let mut s = Geo::new();
			s.pos = pos;
			s.shape = Shape::Sphere(size);
			s
		}

		pub fn from_mesh(mesh:Polymesh) -> Self {
			let mut g = Geo::new();
			// create material slots based on the number of ids on the mesh
			g.materials = vec![0; mesh.num_materials];
			g.shape = Shape::Mesh(mesh);
			g
		}

		pub fn translate(mut self, v:Vector) -> Self
		{
			self.pos += v;
			self
		}
		
		pub fn assign_material(mut self, slot:usize, mtl:usize) -> Self
		{
			if slot < self.materials.len() {
				self.materials[slot] = mtl;
			}
			//*mtl_slot = mtl;
			//else {println!("Material slot out of range {slot}");}
			self
		}

		pub fn calculate(&self, ray:&Ray) -> Option<Hitresult> {
			match self.shape {
				Shape::Sphere(s) 	=> self.calulate_sphere(ray,s),
				Shape::Plane(p,n) 	=> self.calculate_plane(ray,p,n),
				Shape::Mesh(ref m)	=> self.calculate_mesh(ray,m),
			}
		}
		pub fn calulate_sphere(&self, ray:&Ray, size:f32) -> Option<Hitresult> {
			let l = ray.origin - self.pos;
			let a = ray.dir.dot(ray.dir);
			let b = 2.*ray.dir.dot(l);
			let c = l.dot(l) - size * 2.;
			let quad: Option<(f32,f32) >= Geo::solve_quadratic(a,b,c);
			let (mut t0, mut t1) = match quad {
				None 		=> return None,
				Some((x,y)) => (x,y)
			};
			if t0 > t1 {mem::swap(&mut t0, &mut t1);}
			if t0 < 0. {
				t0 = t1;
				if t0 < 0. { return None;}
			}
			let t = t0;
			let hit_pos = ray.origin + ray.dir * t;
			let hit_normal = (hit_pos - self.pos).norm();
			let d = (self.pos - hit_pos).norm();
			let u = 0.5 + d.z.atan2(d.x) / TWO_PI;
			let v = 0.5 - d.y.asin() / PI;
			let result = Hitresult::new(hit_pos, hit_normal, ray.origin, Vec2::new(u,v), Vector::zero(), self.id, self, 0);
			Some(result)
		}
		pub fn calculate_plane(&self, ray:&Ray, pos: Vector, normal:Vector) -> Option<Hitresult> {
			let n = -normal;
			let denom = n.dot(ray.dir.norm());
			if denom > 1e-6
			{
				let p010 = (pos - ray.origin).norm();
				let t = p010.dot(n) / denom;
				if t >= 0. {
					let hit_pos = ray.origin + ray.dir * t;
					let hit = Hitresult::new( hit_pos, normal, ray.origin, Vec2::zero(), Vector::zero(), self.id, self, 0);
					return Some(hit);
				}
			}
			None
		}
		pub fn calculate_mesh(&self, ray:&Ray, mesh:&Polymesh) -> Option<Hitresult> {
			//let hit = mesh.iterate_faces(ray);
			let hit = mesh.cast_ray_with_bvh(ray);
			match hit{
				None => return None,
				Some((p,n,uv, t, mtl_id)) => return Some(Hitresult::new(p,n,ray.origin,uv,t,self.id, self, mtl_id)),
			}
		}

        pub fn get_barycentric(p:Vector, a:Vector, b:Vector, c:Vector) -> (f32, f32, f32) {
            let v0 = b - a;
            let v1 = c - a;
            let v2 = p - a;
            let d00 = v0.dot(v0);
            let d01 = v0.dot(v1);
            let d11 = v1.dot(v1);
            let d20 = v2.dot(v0);
            let d21 = v2.dot(v1);
            let denom = d00 * d11 - d01 * d01;
            let v = (d11 * d20 - d01 * d21) / denom;
            let w = (d00 * d21 - d01 * d20) / denom;
            let u = 1.0 - v - w;
            (v,w,u)
        }

       pub fn solve_quadratic(a:f32, b:f32, c:f32) -> Option<(f32,f32)> {
            let discr = b * b - 4.0 * a * c;
            let mut x0;
            let mut x1;
            if discr < 0. {return None;}
            else if discr == 0. {
                x0 = -0.5 * b / a;
                x1 = x0;
            }
            else {
                let q = if b>0. {-0.5 * (b + discr.sqrt())} else {-0.5 * (b - discr.sqrt())};
                x0 = q / a;
                x1 = c / q;
            }
            if x0 > x1 {mem::swap(&mut x0, &mut x1);}
            Some((x0,x1))
        }

        pub fn interpolate_barycentric(a:Color, b:Color, c:Color, u:f32, v:f32, w:f32) -> Color {
            a * u + b * v + c * w
        }

    }
















// obj parser
// TODO pack into module
pub mod OBJ_Importer
{
	use super::*;

    fn parse_row((_,s):(usize,&str),
		verts:&mut Vec<Vector>,
		faces:&mut Vec<Face>,
		uvs:&mut Vec<Vec2>,
		normals:&mut Vec<Vector>,
		material_map: &mut HashMap<String,usize>,
		current_material: &mut usize ) 
		{
        let mut tokens = s.split_whitespace();
        let first = tokens.next();
        match first {
            Some("v")         => parse_position(&mut tokens, verts),
            Some("vn")        => parse_normal(&mut tokens, normals),
            Some("vt") 	      => parse_uv(&mut tokens, uvs),
            Some("f")   	  => create_face(&mut tokens, faces, current_material),
            Some("usemtl")    => parse_material_id(&mut tokens, material_map, current_material),
            _                 => (),
        };
    }
    fn parse_position(tokens: &mut SplitWhitespace, verts:&mut Vec<Vector>) {
        if let Some(x) = tokens.next(){
            if let Some(y) = tokens.next(){
                 if let Some(z) = tokens.next(){
                    let p = Vector::new(x.parse().unwrap(),y.parse().unwrap(),z.parse().unwrap());
                    verts.push(p);
                }
            }
        }
    }
    fn parse_normal(tokens: &mut SplitWhitespace, normals:&mut Vec<Vector>) {
        if let Some(x) = tokens.next(){
            if let Some(y) = tokens.next(){
                 if let Some(z) = tokens.next(){
                    let mut n = Vector::new(x.parse().unwrap(),y.parse().unwrap(),z.parse().unwrap());
                    normals.push(n);
                }
            }
        }
    }
    fn parse_uv(tokens: &mut SplitWhitespace, uvs:&mut Vec<Vec2>) {
        if let Some(x) = tokens.next(){
            if let Some(y) = tokens.next(){
				let p = Vec2::new(x.parse().unwrap(),y.parse().unwrap());
				//println!("{:?}",p );
				uvs.push(p);
            }
        }
    }




    fn parse_material_id(tokens: &mut SplitWhitespace, materials:&mut HashMap<String,usize>, current_index: &mut usize)
	{
        if let Some(x) = tokens.next(){
			let name : String = x.parse().unwrap();
			let entry = materials.get(&name);
			match entry {
				Some(&index) => *current_index = index,
				None        => {
					let index = materials.len();
					println!("mtl index {} -> from material {name}", index);
					materials.insert(name,index);
					*current_index = index;
				}
			}
        }
    }

    fn create_face(tokens: &mut SplitWhitespace, faces:&mut Vec<Face>, current_material: &usize) {
        if let Some(mut f) = face_from_indices(tokens) {
			f.material_id = *current_material;
			//println!("added face with id {:?}", f.material_id);
			faces.push(f);
		}   
    }

    fn face_from_indices(tokens: &mut SplitWhitespace) -> Option<Face> {
        let mut verts:Vec<usize>   = Vec::with_capacity(3);
        let mut uvs:Vec<usize>     = Vec::with_capacity(3);
        let mut normals:Vec<usize> = Vec::with_capacity(3);

        for word in tokens {
            let mut digits = word.split("/");
            let v  = digits.nth(0);
            let uv = digits.nth(0);
			let n  = digits.nth(0);
            //println!("v: {:?}   uv: {:?}",v,uv );
            if let Some(i) = v {
                let num:usize = i.parse().unwrap();
                verts.push(num);
            }
            if let Some(i) = uv {
            	let num:usize = i.parse().unwrap();
            	uvs.push(num);
            }  
            if let Some(i) = n {
            	let num:usize = i.parse().unwrap();
            	normals.push(num);
            }  
        }
		let out_verts   = (verts[0] - 1, verts[1] - 1, verts[2] - 1);
		let out_normals = (normals[0] - 1, normals[1] - 1, normals[2] - 1);
		let out_uvs = if verts.len() >= 3 { (uvs[0] - 1, uvs[1] - 1, uvs[2] - 1)} else {(0,0,0)};

        if verts.len() >= 3
		{ 
        	return Some(Face::new(out_verts, out_normals, out_uvs));
        }
        None
    }



    pub fn open(filename: &str) -> Option<Polymesh>
    {
        let mut f = File::open(filename).unwrap_or_else(|_| panic!("file not found {filename}"));
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .expect("something went wrong reading the file");

        let rows = contents.lines().enumerate();
        let mut verts:Vec<Vector>   = Vec::new();
        let mut faces:Vec<Face>     = Vec::new();
        let mut uvs:Vec<Vec2>       = Vec::new();
		let mut normals:Vec<Vector> = Vec::new();

		let mut material_map: HashMap<String,usize> = HashMap::new();
		let mut current_material = 0;

        for row in rows
		{
			parse_row(row, 
				&mut verts,
				&mut faces,
				&mut uvs,
				&mut normals,
				&mut material_map,
				&mut current_material
			);
		}
        let mut m = Polymesh::new();
	    m.faces = faces;
	    m.uvs = uvs;
	    m.vertices = verts;
		m.vnormals = normals;
       	//println!("{:?}", m.vnormals);
		
		// check if we actually found some material assignments 
		if material_map.len() > 0 {
			m.num_materials = material_map.len();
		}
	    Some(m)
    }
}