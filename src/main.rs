use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
mod geometry;
use geometry::Light;
use geometry::Sphere;
use geometry::Vec3f;
use geometry::Material;
use std::fs::File;

const WIDTH: usize = 1024;
const HEIGHT: usize = 768;
const FOV: usize = (std::f32::consts::PI / 2.) as usize;
const BACKGROUND: Vec3f = Vec3f(0.2, 0.7, 0.8);

fn scene_intersect<'a>(
    orig: &Vec3f,
    dir: &Vec3f,
    spheres: &[Sphere<'a>],
    hit: &mut Vec3f,
    N: &mut Vec3f,
    material: &'a mut Material<'a>,
) -> bool {
    let mut spheres_dist = std::f32::MAX;
    for sphere in spheres.iter() {
        let mut dist_i: &mut f32 = &mut std::f32::MAX;
        if sphere.ray_intersect(&orig, &dir, &mut dist_i) && *dist_i < spheres_dist {
            spheres_dist = *dist_i;
            *hit = &(orig + dir) * *dist_i;
            *N = &*hit - sphere.center;
            N.normalize();
            *material = sphere.material;
        }
    }

    spheres_dist < 1000.
}

fn cast_ray(orig: &Vec3f, dir: &Vec3f, spheres: &[Sphere], lights: &[Light]) -> Vec3f {
    let mut point: Vec3f = Vec3f(0., 0., 0.);
    let mut N: Vec3f = Vec3f(0., 0., 0.);
    let mut material: Material = Material{diffuse_color: &Vec3f(0., 0., 0.)};
    if !scene_intersect(&orig, &dir, &spheres, &mut point, &mut N, &mut material) {
        return BACKGROUND;
    }
    let mut diffuse_light_intensity = 0.;
    for light in lights.iter() {
        let mut light_dir = light.position - &point;
        light_dir.normalize();
        diffuse_light_intensity += light.intensity * (0.0f32).max(&light_dir * &N);
    }

    // TODO - eliminate warning
    &*material.diffuse_color * diffuse_light_intensity
}

fn render(spheres: &[Sphere], framebuffer: &mut [Vec3f], lights: &[Light]) {
    for j in 0..HEIGHT {
        for i in 0..WIDTH {
            let x = (2. * (i as f32 + 0.5) / WIDTH as f32 - 1.)
                * (FOV as f32 / 2.).tan()
                * (WIDTH as f32)
                / (HEIGHT as f32);
            let y = -(2. * (j as f32 + 0.5) / HEIGHT as f32 - 1.) * (FOV as f32 / 2.).tan();
            let mut dir = Vec3f(x, y, -1.);
            dir.normalize();
            framebuffer[i + j * WIDTH] = cast_ray(&Vec3f(0., 0., 0.), &dir, spheres, lights);
        }
    }
}

fn main() {
    let mut framebuffer = vec![Vec3f(0., 0., 0.); WIDTH * HEIGHT];
    let ivory = Material{ diffuse_color: &Vec3f(0.4, 0.4, 0.3)};
    let red_rubber = Material{ diffuse_color: &Vec3f(0.3, 0.1, 0.1)};
    let spheres = [
        Sphere {
            center: &Vec3f(-3., -0., -16.),
            radius: 2.,
            material: ivory,
        },
        Sphere {
            center: &Vec3f(-1.0, -1.5, -12.),
            radius: 2.,
            material: red_rubber,
        },
        Sphere {
            center: &Vec3f(1.5, -0.5, -18.),
            radius: 3.,
            material: red_rubber,
        },
        Sphere {
            center: &Vec3f(7., 5., -18.),
            radius: 4.,
            material: ivory,
        },
    ];
    let lights = [Light{
        position: &Vec3f(-20., 20., 20.),
        intensity: 1.5,
    }];
    render(&spheres, &mut framebuffer, &lights);
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
