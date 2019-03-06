use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
mod geometry;
use geometry::Light;
use geometry::Material;
use geometry::Sphere;
use geometry::Vec2f;
use geometry::Vec3f;
use std::fs::File;

const WIDTH: usize = 1024;
const HEIGHT: usize = 768;
const FOV: usize = (std::f32::consts::PI / 2.) as usize;
const BACKGROUND: Vec3f = Vec3f(0.2, 0.7, 0.8);

fn reflect(I: &Vec3f, N: &Vec3f) -> Vec3f {
    *I - *N * 2. * (*I * *N)
}

fn scene_intersect<'a>(
    orig: &Vec3f,
    dir: &Vec3f,
    spheres: &[Sphere<'a>],
    hit: &mut Vec3f,
    N: &mut Vec3f,
) -> Option<Material<'a>> {
    let mut spheres_dist = std::f32::MAX;
    let mut material: Material = Default::default();
    for sphere in spheres.iter() {
        let mut dist_i: &mut f32 = &mut std::f32::MAX;
        if sphere.ray_intersect(&orig, &dir, &mut dist_i) && *dist_i < spheres_dist {
            spheres_dist = *dist_i;
            *hit = (*orig + *dir) * *dist_i;
            *N = *hit - *sphere.center;
            N.normalize();
            material = sphere.material;
        }
    }

    if spheres_dist < 1000. {
        Some(material)
    } else {
        None
    }
}

fn cast_ray(orig: &Vec3f, dir: &Vec3f, spheres: &[Sphere], lights: &[Light]) -> Vec3f {
    let mut point: Vec3f = Vec3f(0., 0., 0.);
    let mut N: Vec3f = Vec3f(0., 0., 0.);
    match scene_intersect(&orig, &dir, &spheres, &mut point, &mut N) {
        None => BACKGROUND,
        Some(material) => {
            let mut diffuse_light_intensity = 0.;
            let mut specular_ligth_intensity = 0.;
            for light in lights.iter() {
                let mut light_dir = *light.position - point;
                light_dir.normalize();
                let light_distance = (*light.position - point).norm();
                let ldn = light_dir * N;
                let shadow_orig = if ldn < 0. {
                    point - N * 1e-3
                } else {
                    point + N * 1e-3
                };
                let mut shadow_pt: Vec3f = Default::default();
                let mut shadow_N: Vec3f = Default::default();
                if scene_intersect(
                    &shadow_orig,
                    &light_dir,
                    &spheres,
                    &mut shadow_pt,
                    &mut shadow_N,
                )
                .is_some()
                    && (shadow_pt - shadow_orig).norm() < light_distance
                {
                    continue;
                }

                diffuse_light_intensity += light.intensity * (0.0f32).max(ldn);

                specular_ligth_intensity += {
                    let a = reflect(&-light_dir, &N) * *dir;
                    let b = (0.0f32).max(-a);
                    let c = b.powf(material.specular_exponent);
                    c * light.intensity
                }
            }

            *material.diffuse_color * diffuse_light_intensity * material.albedo.0
                + Vec3f(1., 1., 1.) * specular_ligth_intensity * material.albedo.1
        }
    }
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
    let ivory = Material {
        albedo: &Vec2f(0.6, 0.3),
        diffuse_color: &Vec3f(0.4, 0.4, 0.3),
        specular_exponent: 50.,
    };
    let red_rubber = Material {
        albedo: &Vec2f(0.9, 0.1),
        diffuse_color: &Vec3f(0.3, 0.1, 0.1),
        specular_exponent: 10.,
    };
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
    let lights = [
        Light {
            position: &Vec3f(-20., 20., 20.),
            intensity: 1.5,
        },
        Light {
            position: &Vec3f(30., 50., -25.),
            intensity: 1.8,
        },
        Light {
            position: &Vec3f(30., 20., 30.),
            intensity: 1.7,
        },
    ];
    render(&spheres, &mut framebuffer, &lights);
    let path = Path::new("./out.ppm");
    let file = File::create(&path).expect("Cannot create out.ppm");
    let mut stream = BufWriter::new(file);
    write!(stream, "P6\n{} {}\n255\n", WIDTH, HEIGHT).expect("Error writing to file");
    for i in 0..(WIDTH * HEIGHT) {
        let zero = 0.0f32;
        let one = 1.0f32;
        let mut c = framebuffer[i];
        let max = (c.0).max((c.1).max(c.2));
        if max > 1.0 {
            c = c * (1.0 / max);
        }
        let Vec3f(x, y, z) = c;
        for j in &[x, y, z] {
            let pixel = (255.0f32 * zero.max(one.min(*j))) as u8;
            stream.write(&[pixel]).expect("Cannot write to file");
        }
    }
}
