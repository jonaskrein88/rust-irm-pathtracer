use super::*;

use std::fs::File;
use std::str::{SplitWhitespace, Lines};
use std::io::Read;







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




#[derive(Debug)]
pub struct Face {
    a:usize,
    b:usize,
    c:usize,
    uva:usize,
    uvb:usize,
    uvc:usize,
}
impl Face {
    fn new(a:usize, b:usize, c:usize) -> Face {
        Face{a:a, b:b, c:c, uva:0, uvb:0, uvc:0}
    }
    fn new_with_uvs(a:usize, b:usize, c:usize, uva:usize, uvb:usize, uvc:usize) -> Face {
        let mut f = Face::new(a,b,c);
        f.add_uvs(uva,uvb,uvc);
        f
    }
    fn add_uvs(&mut self, a:usize, b:usize, c:usize) {
        self.uva = a;
        self.uvb = b;
        self.uvc = c;
    }
}






	pub struct Points(Vector, Vector, Vector);


	pub struct Polymesh {
		faces: Vec<Face>,
		vertices: Vec<Vector>,
		normals: Vec<Vector>,
		aabb:(Vector,Vector),
		has_uvs:bool,
		uvs: Vec<Vec2>,
		vnormals: Vec<Vector>,
		ftangents: Vec<Vector>,
		vtangents: Vec<Vector>,
	}
	impl Polymesh
	{
		pub fn new() -> Polymesh {
			Polymesh { 
				faces:Vec::new(), 
				vertices:Vec::new(),
				normals:Vec::new(),
				aabb:(Vector::empty(),Vector::empty()),
				uvs:Vec::new(),
				vnormals:Vec::new(),
				has_uvs:false,
				ftangents: Vec::new(),
				vtangents: Vec::new(),
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
		fn prepass(&mut self) {
			self.pre_calculate_aabb();
			if self.uvs.len() > 0 {self.has_uvs = true}
			self.pre_calculate_normals();
			self.calculate_vertex_normals();
		}
		pub fn add_face(&mut self, face:Face) {self.faces.push(face);}

		pub fn add_plane(&mut self) {

			let v0 = Vector::new(50.,-10., 50.);    
			let v1 = Vector::new(50.,-10., -50.);
			let v2 = Vector::new(-50.,-10., -50.);
			let v3 = Vector::new(-50.,-10., 50.);

			let uv0 = self.add_uv(Vec2::new(0.,0.));
			let uv1 = self.add_uv(Vec2::new(0.,1.));
			let uv2 = self.add_uv(Vec2::new(1.,1.));
			let uv3 = self.add_uv(Vec2::new(1.,0.));

			let i0 = self.add_point(v0);
			let i1 = self.add_point(v1);
			let i2 = self.add_point(v2);
			let i3 = self.add_point(v3);
			let f1 = Face::new_with_uvs(i0,i1,i2, uv0, uv1, uv2);
			let f2 = Face::new_with_uvs(i2,i3,i0, uv2, uv3, uv0);
			self.add_face(f1);
			self.add_face(f2);
		}
		fn add_point(&mut self, v:Vector) -> usize {
			self.vertices.push(v);
			self.vertices.len() - 1
		}
		fn add_uv(&mut self, uv:Vec2) -> usize {
			self.uvs.push(uv);
			self.uvs.len() - 1
		}

		pub fn transform_uvs(&mut self, s:f32, t:f32) {
			for uv in &mut self.uvs {
				uv.x = uv.x * s;
				uv.y = uv.y * t;
			}
			println!("\nuvs{:?}",self.uvs );
		}

		pub fn transform(&mut self, m:Matrix) {
			for v in &mut self.vertices {
				*v = v.transform(&m);
			}
		}

		fn calculate_vertex_normals(&mut self) {
			// normals
			for i in 0..self.vertices.len(){
				let faces = self.faces_from_point(i);
				let vnormal = self.vnormal_from_fnormals(faces);
				self.vnormals.push(vnormal);
			}
			// tangents
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
		}
		fn faces_from_point(&self, v:usize) -> Vec<usize> {
			let mut out:Vec<usize> = Vec::new();
			for i in 0..self.faces.len() {
				let face = &self.faces[i];
				if face.a == v || face.b == v || face.c == v { out.push(i);}
			}
			out
		}
		fn vnormal_from_fnormals(&self, face_ids:Vec<usize>) -> Vector {
			let mut out_normal = Vector::empty();
			for i in 0..face_ids.len() {
				let fnormal = self.normals[face_ids[i]];
				out_normal = out_normal + fnormal;
			}
			out_normal.norm()
		}
		pub fn translate(&mut self, t:Vector) {
			for v in &mut self.vertices {
				*v = v.add(t);
			}
		}
		pub fn scale(&mut self, f:f32) {
			let m = Matrix::scale(f);
			self.transform(m);
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
		fn iterate_faces(&self, ray:&Ray) -> Option<(Vector, Vector, Vec2, Vector)> {
			if !self.calculate_aabb_intersection(ray) { return None;}
			let mut best_dist = f32::MAX;
			let mut best_hit: Option<(Vector,&Face,f32,f32,f32)> = None;
			for i in 0..self.faces.len() {
				let hit = self.calculate_hit(ray, &self.faces[i], &self.normals[i]);
				match hit {
					None => continue,
					Some((pos,..)) => {
						let dist = (ray.origin - pos).length();
						if dist < best_dist {
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
					let uv = if self.has_uvs {self.interpolate_uv(f,(u,v,w))} else {Vec2::new(0.,0.)};
					let tangent = if self.has_uvs {self.interpolate_tangent(f,(u,v,w))} else {Vector::new(1.,0.,0.)};
					Some((pos, normal, uv, tangent))
				},
			}
		}
		fn calculate_hit<'a>(&self, ray: &Ray, f:&'a Face, n:&Vector) -> Option<(Vector, &'a Face,f32,f32,f32)> {
			if n.dot(ray.dir) > 0. { return None;}
			// Möller-Trumbore
			let p = self.get_positions(f);
			let k_epsilon = 0.001;
			let v0v1 = p.1 - p.0;
			let v0v2 = p.2 - p.0;
			let pvec = ray.dir.cross(v0v2); 
			let det = v0v1.dot(pvec);
			if det < k_epsilon {return None}; 
			let inv_det = 1. / det;
			let tvec = ray.origin - p.0;
			let u = tvec.dot(pvec) * inv_det;
			if u < 0. || u > 1. {return None;}
			let qvec = tvec.cross(v0v1);
			let v = ray.dir.dot(qvec) * inv_det;
			if v < 0. || u + v > 1. { return None;}
			let t = v0v2.dot(qvec) * inv_det;
			let hit_pos = ray.origin + ray.dir.norm() * t;
			let w = 1. - u - v;
			Some((hit_pos, f, u,v,w))
			//let normal = Vector::new(0.,1.,0.);
			//let tangent = Vector::new(0.,1.,0.);
			//let uv = Vec2::new(1.,0.);
			//let normal = self.interpolate_normal(f,(u,v,w));
			//let uv = if self.has_uvs {self.interpolate_uv(f,(u,v,w))} else {Vec2::new(0.,0.)};
			//let tangent = if self.has_uvs {self.interpolate_tangent(f,(u,v,w))} else {Vector::new(1.,0.,0.)};
			//Some((hit_pos, normal, uv, tangent))
		}
		fn pre_calculate_normals(&mut self) {
			for f in &self.faces {
				let points = self.get_positions(f);
				let n = self.calculate_normal(&points);
				self.normals.push(n);
				let t = self.calculate_tangent(&f);
				self.ftangents.push(t);
			}
		}
		fn calculate_normal(&self, p:&Points) -> Vector {
			let e1 = p.1 - p.0;
			let e2 = p.2 - p.0;
			e1.cross(e2).norm()
		}
		fn interpolate_normal(&self, f:&Face, (u,v,w):(f32,f32,f32)) -> Vector {
			let c = self.vnormals[f.a];
			let a = self.vnormals[f.b];
			let b = self.vnormals[f.c];
			a * u + b * v + c * w
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
		fn calculate_tangent(&self, f:&Face) -> Vector {
			let Points(v0,v1,v2) = self.get_positions(f);
			let uv0 = self.uvs[f.uva];
			let uv1 = self.uvs[f.uvb];
			let uv2 = self.uvs[f.uvc];
			let delta1 = v1 - v0;
			let delta2 = v2 - v0;
			let delta_uv_1 = uv1 - uv0;
			let delta_uv_2 = uv2 - uv0;
			let r = 1. / (delta_uv_1.x * delta_uv_2.y - delta_uv_1.y * delta_uv_2.x);
			let tangent = (delta1 * delta_uv_2.y - delta2 * delta_uv_1.y) * r;
			tangent.norm()
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
		pub material: Material,
		pub shape: Shape,
	}

	impl Geo {
		pub fn new() -> Geo{ 
			Geo {
				id:0,
				pos:Vector::empty(),
				shape:Shape::Sphere(1.0),
				material:Material::new(),
			}
		}
		pub fn prepass(&mut self) {
			if let Shape::Mesh(ref mut m) = self.shape { m.prepass();}
		}
		pub fn create_sphere(pos: Vector) -> Geo {
			let mut s = Geo::new();
			s.pos = pos;
			s.set_size(0.2);
			s
		}
		pub fn create_mesh(mesh:Polymesh) -> Geo {
			let mut g = Geo::new();
			g.shape = Shape::Mesh(mesh);
			g
		}
		pub fn set_size(&mut self,s: f32) {
			self.shape = Shape::Sphere(s);
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
			let result = Hitresult::new_with_uvs(hit_pos, hit_normal,Vec2::new(u,v), self.id, self);
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
					let hit = Hitresult::new( hit_pos, normal, self.id, self);
					return Some(hit);
				}
			}
			None
		}
		pub fn calculate_mesh(&self, ray:&Ray, mesh:&Polymesh) -> Option<Hitresult> {
			let hit = mesh.iterate_faces(ray);
			match hit{
				None => return None,
				Some((p,n,uv, t)) => return Some(Hitresult::new_with_tan(p,n,uv,t, self.id, self)),
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


    fn parse_row((_,s):(usize,&str), mut verts:&mut Vec<Vector>, mut faces:&mut Vec<Face>, mut uvs:&mut Vec<Vec2>) {
        let mut tokens = s.split_whitespace();
        let first = tokens.next();
        match first {
            Some("v")   => parse_vector(&mut tokens, &mut verts),
            Some("vt") 	=> parse_uv(&mut tokens, &mut uvs),
            Some("f")   => create_face(&mut tokens, &mut faces),
            _           => (),
        };
    }
    fn parse_vector(tokens: &mut SplitWhitespace, verts:&mut Vec<Vector>) {
        if let Some(x) = tokens.next(){
            if let Some(y) = tokens.next(){
                 if let Some(z) = tokens.next()
                 {
                    let p = Vector::new(x.parse().unwrap(),y.parse().unwrap(),z.parse().unwrap());
                    verts.push(p);
                }
            }
        }
    }
    fn parse_uv(tokens: &mut SplitWhitespace, uvs:&mut Vec<Vec2>) {
        if let Some(x) = tokens.next(){
            if let Some(y) = tokens.next(){
                 //if let Some(z) = tokens.next()
                 //{
                    let p = Vec2::new(x.parse().unwrap(),y.parse().unwrap());
                    //println!("{:?}",p );
                    uvs.push(p);
                //}
            }
        }
    }
    fn create_face(tokens: &mut SplitWhitespace, faces:&mut Vec<Face>) {
        if let Some(f) = parse_indexes(tokens) {faces.push(f);}   
    }
    fn parse_indexes(tokens: &mut SplitWhitespace) -> Option<Face> {
        let mut verts:Vec<usize> = Vec::with_capacity(3);
        let mut uvs:Vec<usize> = Vec::with_capacity(3);
        for word in tokens {
            let mut digits = word.split("/");
            let v = digits.nth(0);
            let uv = digits.nth(0);
            //println!("v: {:?}   uv: {:?}",v,uv );
            if let Some(i) = v {
                let num:usize = i.parse().unwrap();
                verts.push(num);
            }
             if let Some(i) = uv {
                let num:usize = i.parse().unwrap();
                uvs.push(num);
            }  
        }
        if verts.len() >= 3 { 
        	if uvs.len() == 3 { return Some(Face::new_with_uvs(verts[0] - 1, verts[1] - 1, verts[2] - 1, uvs[0] - 1, uvs[1] - 1, uvs[2] - 1))}
            else {return Some(Face::new(verts[0] - 1, verts[1] - 1, verts[2] - 1));}
        }
        None
    }
    pub fn open(filename: &str) -> Polymesh
    {
        let mut f = File::open(filename).expect("file not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .expect("something went wrong reading the file");

        let rows = contents.lines().enumerate();
        let mut verts:Vec<Vector> = Vec::new();
        let mut faces:Vec<Face> = Vec::new();
        let mut uvs:Vec<Vec2> = Vec::new();

        for l in rows { parse_row(l, &mut verts, &mut faces, &mut uvs);}
       	//println!("{:?}", uvs);
        let mut m = Polymesh::new();
	    m.faces = faces;
	    m.uvs = uvs;
	    m.vertices = verts;
	    m
    }
