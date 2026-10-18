use super::*;
use std::ops::{Add, Sub, Mul, Neg, Div, AddAssign};





#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r:f32,
    pub g:f32,
    pub b:f32,
}
impl Color 
{
    pub fn new(x:f32, y:f32, z:f32) -> Color {Color{r:x, g:y, b:z}}
    pub fn new_mono(x:f32) -> Color {Color::new(x,x,x)}

    pub fn black() -> Color {Color::new(0.,0.,0.)}
    pub fn white() -> Color {Color::new(1.,1.,1.)}
    pub fn grey() -> Color {Color::new(0.5,0.5,0.5)}
    pub fn light_grey() -> Color {Color::new(0.85,0.85,0.85)}
    pub fn red() -> Color {Color::new(1.,0.,0.)}
    pub fn blue() -> Color {Color::new(0.,0.,1.)}
    pub fn green() -> Color {Color::new(0.,1.,0.)}
    pub fn yellow() -> Color {Color::new(1.,1.,0.)}

    pub fn add(&self, c:Color) -> Color {
        Color::new(self.r + c.r, self.g + c.g, self.b + c.b)
    }
    pub fn mult(&self, v:f32) -> Color {
        Color::new(self.r * v, self.g * v,  self.b * v)
    }
    pub fn pow(&self, f:f32) -> Color {
        let r = self.r.powf(f);
        let g = self.g.powf(f);
        let b = self.b.powf(f);
        Color::new(r,g,b)
    }
    pub fn to_0_255(&self) -> (u8,u8,u8) {
        let c = self.apply_gamma(2.20);
        let x = (c.r * 255.).max(0.).min(255.) as u8;
        let y = (c.g * 255.).max(0.).min(255.) as u8;
        let z = (c.b * 255.).max(0.).min(255.) as u8;
        (x,y,z)
    }
    pub fn apply_gamma(&self, gamma:f32) -> Color {
        self.pow(1./gamma)
    }
    pub fn new_from_255(x:u8, y:u8, z:u8) -> Color {
        let c = Color{r:x as f32, g:y as f32, b:z as f32};
        c / 255.
    }
    pub fn interpolate(&self, c: Color, f:f32) -> Color {
        //let x = f.max(0.).min(1.);
        *self * f + c * (1. - f)
    }
    pub fn to_vector(&self) -> Vector {
        let c = *self * 2.;
        Vector::new(c.r - 1., c.g - 1., c.b - 1.)
    }
    pub fn clamp(&self, v:f32) -> Color {
        Self::new(self.r.min(v), self.g.min(v), self.b.min(v))

    }
}



impl Add for Color {
    type Output = Color;
    fn add(self, v: Color) -> Color {
        Color{r: self.r + v.r, g: self.g + v.g, b: self.b + v.b}
    }
}
impl AddAssign for Color {
    fn add_assign(&mut self, v: Color) {
        *self = Color{r: self.r + v.r, g: self.g + v.g, b: self.b + v.b};
    }
}
impl Sub for Color {
    type Output = Color;
    fn sub(self, v: Color) -> Color {
        Color{r: self.r - v.r, g: self.g - v.g, b: self.b - v.b}
    }
}
impl Mul<f32> for Color {
    type Output = Color;
    fn mul(self, v:f32) -> Color {
        Color::new(self.r * v, self.g * v, self.b * v)
    } 
}
impl Mul for Color {
    type Output = Color;
    fn mul(self, c:Color) -> Color {
        Color::new(self.r * c.r, self.g * c.g, self.b * c.b)
    } 
}
impl Div<f32> for Color {
    type Output = Color;
    fn div(self, v:f32) -> Color {
        Color::new(self.r / v, self.g / v, self.b / v)
    } 
}
impl PartialEq for Color {
    fn eq(&self, other:&Color) -> bool {
        let bias = 0.1;
        (self.r - other.r).abs() < bias && (self.g - other.g).abs() < bias && (self.b - other.b).abs() < bias
    }
}
