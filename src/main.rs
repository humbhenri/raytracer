use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::ops;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
struct Vec3f(f32, f32, f32);

impl ops::Sub for &Vec3f {
    type Output = Vec3f;
    fn sub(self, _rhs: &Vec3f) -> Vec3f {
        let Vec3f(x, y, z) = self;
        let Vec3f(_x, _y, _z) = _rhs;
        Vec3f(x - _x, y - _y, z - _z)
    }
}

impl ops::Mul for &Vec3f {
    type Output = f32;
    fn mul(self, _rhs: &Vec3f) -> f32 {
        let Vec3f(x, y, z) = self;
        let Vec3f(_x, _y, _z) = _rhs;
        x * _x + y * _y + z * _z
    }
}

struct Sphere<'a> {
    center: &'a Vec3f,
    radius: f32,
}

impl<'a> Sphere<'a> {
    fn ray_intersect(&self, orig: &Vec3f, dir: &Vec3f, t0: &'a mut f32) -> bool {
        let L = self.center - orig;
        let tca = &L * dir;
        let d2 = &L * &L - tca.powi(2);
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

fn main() {
    const WIDTH: usize = 1024;
    const HEIGHT: usize = 768;
    let mut framebuffer = vec![Vec3f(0., 0., 0.); WIDTH * HEIGHT];

    for j in 0..HEIGHT {
        for i in 0..WIDTH {
            let pixel = Vec3f(
                (j as f32) / (HEIGHT as f32),
                (i as f32) / (WIDTH as f32),
                0.,
            );

            framebuffer[i + j * WIDTH] = pixel;
        }
    }

    let path = Path::new("./out.ppm");
    let file = File::create(&path).expect("Cannot create out.ppm");
    let mut stream = BufWriter::new(file);
    write!(stream, "P6\n{} {}\n255\n", WIDTH, HEIGHT).expect("Error writing to file");

    for i in 0..(WIDTH * HEIGHT) {
        let zero = 0.0f32;
        let one = 1.0f32;
        let Vec3f(x, y, z) = framebuffer[i];
        for j in &[x, y, z] {
            let pixel = (255.0f32 * zero.max(one.min(*j))) as u8;
            stream.write(&[pixel]).expect("Cannot write to file");
        }
    }
}
