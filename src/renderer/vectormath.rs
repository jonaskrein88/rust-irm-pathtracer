extern crate rand;


use std::{f64,f32};
use std::ops::{Add, Sub, Mul, Neg, Div, AddAssign};
use std::fmt;
use std::mem;
use super::*;



const PI: f32 		= 3.14159265359; 
const TWO_PI: f32 	= 6.2831832; 





#[derive(Copy, Clone)]
pub struct Vector {
    pub x: f32, 
    pub y: f32, 
    pub z: f32,
}
impl Vector 
{	
    pub fn empty() -> Vector { Vector{x: 0., y:0., z:0.}}

    pub fn new(x: f32, y: f32 ,z : f32) -> Vector {Vector{x:x, y:y, z:z}}

    pub fn random() -> Vector {
        let x = rand::random::<f32>();
        let y = rand::random::<f32>();
        let z = rand::random::<f32>();
        Vector::new(x,y,z)
    }
    pub fn add(&self, v: Vector) -> Vector{
        Vector{x: self.x + v.x, y: self.y + v.y, z: self.z + v.z}
    }
    pub fn sub(&self, v: Vector) -> Vector{
        Vector{x: self.x - v.x, y: self.y - v.y, z: self.z - v.z}
    }
    pub fn mult(&self, v:f32) -> Vector {
        Vector{x: self.x * v, y: self.y * v, z: self.z * v}
    }
    pub fn div(&self, v:f32) -> Vector {
        self.mult(1./v)
    }
    pub fn length(&self) -> f32 {
        f32::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }
    pub fn dist(self, v:Vector) -> f32 {
        (self - v).length()
    }
    pub fn norm(&self) -> Vector {
        self.div(self.length())
    }
    pub fn cross(&self, v: Vector) -> Vector{
        let x = self.y * v.z - self.z *v.y;
        let y = self.z * v.x - self.x *v.z;
        let z = self.x * v.y - self.y *v.x;
        Vector::new(x,y,z)
    }
    pub fn dot(&self, v: Vector) -> f32 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }
    pub fn angle_between(&self, v:Vector) -> f32 {
        self.norm().dot(v.norm()).acos()
    }
    pub fn rotate_x(&mut self, f:f32) {
        let mx = Matrix::rotate_x(f);
        mx.apply(self)
    }
    pub fn rotate_y(&mut self, f:f32) {
        let my = Matrix::rotate_y(f);
        my.apply(self)
    }
    pub fn rotate_z(&mut self, f:f32) {
        let mz = Matrix::rotate_z(f);
        mz.apply(self)
    }
    pub fn to_color(&self) -> Color {
        Color::new(self.x, self.y, self.z)
    }
    pub fn to_array(&self) -> [f64; 3] {
        [self.x as f64, self.y as f64, self.z as f64]
    }
    pub fn normal_to_color(&self) -> Color {
        let r = self.x / 2.0 + 0.5;
        let g = self.y / 2.0 + 0.5;
        let b = self.z / 2.0 + 0.5;
        Vector::new(r,g,b).to_color()
    }
    pub fn rotate(&mut self, x:f32, y:f32, z:f32) {
        self.rotate_x(x);
        self.rotate_y(y);
        self.rotate_z(z);
    }
    pub fn transform(&self, m:&Matrix) -> Vector {
        let x = self.x * m.fields[0][0] + self.y * m.fields[0][1] + self.z * m.fields[0][2];
        let y = self.x * m.fields[1][0] + self.y * m.fields[1][1] + self.z * m.fields[1][2];
        let z = self.x * m.fields[2][0] + self.y * m.fields[2][1] + self.z * m.fields[2][2];
        Vector::new(x,y,z)
    }
    pub fn axis_aligned_hemispherical(angle:f32) -> Vector {
        let phi = rand::random::<f32>() * angle;	// PI/2.0;    // 0 - Pi/2
        let theta = rand::random::<f32>() * TWO_PI;    // 0 - Pi*2
        let x = phi.sin() * theta.cos();
        let y = phi.sin() * theta.sin();
        let z = phi.cos();
        Vector::new(x,y,z).norm()
    }
    pub fn random_angle(self, angle:f32) -> Vector {
        let p = Vector::axis_aligned_hemispherical(angle);
        let m = Matrix::from_vector(self);
        p.transform(&m)
    }
    pub fn random_hemispherical(self) -> Vector { self.random_angle(PI / 2.0) }

	pub fn calc_reflect_vector(mut i: Vector, mut n: Vector) -> Vector {
		i = i.norm();
		n = n.norm();
		(i - n * 2. * n.dot(i)).norm()
	}
	pub fn calc_refract_vector(i:Vector, mut n:Vector, ior:f32) -> Vector {
		let mut cosi = i.dot(n).min(1.).max(-1.);
		let mut etai = 1.;
		let mut etat = ior;
		if cosi < 0. {cosi = -cosi}
		else {
			mem::swap(&mut etat, &mut etai);
			n = -n;
		}
		let eta = etai / etat;
		let k = 1.- eta.powi(2) * (1. - cosi.powi(2));
		if k <= 0. {return Vector::empty();}
		i.mult(eta).add(n.mult((eta * cosi) - k.sqrt()))
	}

}

impl Add for Vector {
    type Output = Vector;
    fn add(self, v: Vector) -> Vector {
        Vector{x: self.x + v.x, y: self.y + v.y, z: self.z + v.z}
    }
}
impl AddAssign for Vector {
    fn add_assign(&mut self, v: Vector) {
        *self = Vector{x: self.x + v.x, y: self.y + v.y, z: self.z + v.z};
    }
}
impl Sub for Vector {
    type Output = Vector;
    fn sub(self, v: Vector) -> Vector {
        Vector{x: self.x - v.x, y: self.y - v.y, z: self.z - v.z}
    }
}
impl Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Vector {
        Vector::new(-self.x, -self.y, -self.z)
    } 
}
impl Mul<f32> for Vector {
    type Output = Vector;
    fn mul(self, v:f32) -> Vector {
        Vector::new(self.x * v, self.y * v, self.z * v)
    } 
}
impl fmt::Debug for Vector {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "v: {0:.2}, {1:.2}, {2:.2}",  self.x, self.y, self.z)
    }
}
impl PartialEq for Vector {
    fn eq(&self, other:&Vector) -> bool {
        let bias = 0.1;
        (self.x - other.x).abs() < bias && (self.y - other.y).abs() < bias && (self.z - other.z).abs() < bias
    }
}












#[derive(Copy, Clone, Debug)]
pub struct Vec2{
    pub x:f32,
    pub y:f32,
}
impl Vec2 { 
    pub fn new(x:f32, y:f32) -> Vec2 {
        Vec2{x:x, y:y}
    }
    pub fn to_color(&self) -> Color {Color::new(self.x % 1.0, self.y % 1.0, 0.0)}
}
impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, v:f32) -> Vec2 {
        Vec2::new(self.x * v, self.y * v)
    } 
}
impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, v: Vec2) -> Vec2 {
        Vec2{x: self.x + v.x, y: self.y + v.y}
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, v: Vec2) -> Vec2 {
        Vec2{x: self.x - v.x, y: self.y - v.y}
    }
}



#[derive(Debug)]
	pub struct Matrix {
		fields: [[f32; 4]; 4],
	}
	impl Matrix {
		pub fn new() -> Matrix {
			let zeros: [[f32; 4]; 4] = [[0f32; 4]; 4];
			Matrix{fields: zeros}
		}
		pub fn from_vectors(forward:Vector, up:Vector, right:Vector) -> Matrix {
			let mut m = Matrix::new();
			m.fields[0][0] = right.x;
		   	m.fields[1][0] = right.y;
		   	m.fields[2][0] = right.z;
		   	m.fields[0][1] = up.x;
		   	m.fields[1][1] = up.y;
		   	m.fields[2][1] = up.z;
		   	m.fields[0][2] = forward.x;
		   	m.fields[1][2] = forward.y;
		   	m.fields[2][2] = forward.z;
		   	m
		}
		pub fn from_vector(v:Vector) -> Matrix {
			let rand = if v == Vector::new(0.,1.,0.) {Vector::new(0.,0.,1.)}
			else { Vector::new(0.,1.,0.)};
			let up = rand.cross(v);
			let right = up.cross(v);
			Matrix::from_vectors(v, up, right)
		}
		pub fn rotate_x(f: f32) -> Matrix {
			let mut new_matrix = Matrix::new();
			new_matrix.fields[0][0] = 1.;
			new_matrix.fields[1][1] = f.cos();
			new_matrix.fields[1][2] = -f.sin();
			new_matrix.fields[2][1] = f.sin();
			new_matrix.fields[2][2] = f.cos();
			new_matrix
		}
		pub fn rotate_y(f: f32) -> Matrix {
			let mut new_matrix = Matrix::new();
			new_matrix.fields[0][0] = f.cos();
			new_matrix.fields[0][2] = f.sin();
			new_matrix.fields[1][1] = 1.;
			new_matrix.fields[2][0] = -f.sin();
			new_matrix.fields[2][2] = f.cos();
			new_matrix
		}
		pub fn rotate_z(f: f32) -> Matrix {
			let mut new_matrix = Matrix::new();
			new_matrix.fields[0][0] = f.cos();
			new_matrix.fields[0][1] = -f.sin();
			new_matrix.fields[1][0] = f.sin();
			new_matrix.fields[1][1] = f.cos();
			new_matrix.fields[2][2] = 1.;
			new_matrix
		}
		pub fn scale(f: f32) -> Matrix {
			let mut new_matrix = Matrix::new();
			new_matrix.fields[0][0] = f;
			new_matrix.fields[1][1] = f;
			new_matrix.fields[2][2] = f;
			new_matrix.fields[3][3] = 1.;
			new_matrix
		}
		pub fn apply(&self, v: &mut Vector) {
			v.x = v.x * self.fields[0][0] + v.y * self.fields[0][1] + v.z * self.fields[0][2];
			v.y = v.x * self.fields[1][0] + v.y * self.fields[1][1] + v.z * self.fields[1][2];
			v.z = v.x * self.fields[2][0] + v.y * self.fields[2][1] + v.z * self.fields[2][2];
		}
	}
