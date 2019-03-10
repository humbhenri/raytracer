use std::ops;

#[derive(Debug, Clone, Copy, Default)]
pub struct Vec3f(pub f32, pub f32, pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct Vec2f(pub f32, pub f32);

impl Vec2f {
    pub const fn new() -> Self {
        Vec2f(0., 0.)
    }
}

impl Vec3f {
    pub fn norm(&self) -> f32 {
        (self.0.powf(2.) + self.1.powf(2.) + self.2.powf(2.)).sqrt()
    }

    pub fn normalize(&mut self) -> &Self {
        let norm = self.norm();
        let Vec3f(x, y, z) = self;
        *x = *x * (1. / norm);
        *y = *y * (1. / norm);
        *z = *z * (1. / norm);
        self
    }

    pub const fn new() -> Self {
        Vec3f(0., 0., 0.)
    }
}

impl ops::Add for Vec3f {
    type Output = Vec3f;
    fn add(self, _rhs: Vec3f) -> Vec3f {
        let Vec3f(x, y, z) = self;
        let Vec3f(_x, _y, _z) = _rhs;
        Vec3f(x + _x, y + _y, z + _z)
    }
}

impl ops::Sub for Vec3f {
    type Output = Vec3f;
    fn sub(self, _rhs: Vec3f) -> Vec3f {
        let Vec3f(x, y, z) = self;
        let Vec3f(_x, _y, _z) = _rhs;
        Vec3f(x - _x, y - _y, z - _z)
    }
}

impl ops::Mul for Vec3f {
    type Output = f32;
    fn mul(self, _rhs: Vec3f) -> f32 {
        let Vec3f(x, y, z) = self;
        let Vec3f(_x, _y, _z) = _rhs;
        x * _x + y * _y + z * _z
    }
}

impl ops::Mul<f32> for Vec3f {
    type Output = Vec3f;
    fn mul(self, _rhs: f32) -> Vec3f {
        let Vec3f(x, y, z) = self;
        Vec3f(x * _rhs, y * _rhs, z * _rhs)
    }
}

impl ops::Neg for Vec3f {
    type Output = Vec3f;
    fn neg(self) -> Vec3f {
        let Vec3f(x, y, z) = self;
        Vec3f(-x, -y, -z)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse_color: Vec3f,
    pub albedo: Vec3f,
    pub specular_exponent: f32,
}

impl Material {
    pub const fn new() -> Self {
        Material {
            diffuse_color: Vec3f::new(),
            albedo: Vec3f(1., 0., 0.),
            specular_exponent: 0.,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vec3f,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub fn ray_intersect(&self, orig: &Vec3f, dir: &Vec3f, t0: &mut f32) -> bool {
        let L = self.center - *orig;
        let tca = L * *dir;
        let d2 = L * L - tca.powi(2);
        if d2 > self.radius.powi(2) {
            return false;
        } else {
            let thc = (self.radius.powi(2) - d2).sqrt();
            *t0 = &tca - thc;
            let t1 = tca + thc;
            *t0 = if t0 < &mut 0. { t1 } else { *t0 };
            t0 >= &mut 0.
        }
    }
}

pub struct Light {
    pub position: Vec3f,
    pub intensity: f32,
}
