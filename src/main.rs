use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
mod geometry;
use geometry::Light;
use geometry::Material;
use geometry::Sphere;
use geometry::Vec3f;
use geometry::Vec4f;
use std::fs::File;

const WIDTH: usize = 1024;
const HEIGHT: usize = 768;
const FOV: usize = (std::f32::consts::PI / 2.) as usize;
const BACKGROUND: Vec3f = Vec3f(0.2, 0.7, 0.8);

fn reflect(I: &Vec3f, N: &Vec3f) -> Vec3f {
    *I - *N * 2. * (*I * *N)
}

fn refract(I: &Vec3f, N: &Vec3f, refractive_index: f32) -> Vec3f {
    let mut cosi = -(-1.0f32).max(1.0f32.min(*I * *N));
    let mut etai = 1.;
    let mut etat = refractive_index;
    let mut n = *N;
    if cosi < 0. {
        cosi = -cosi;
        std::mem::swap(&mut etai, &mut etat);
        n = -n;
    }
    let eta = etai / etat;
    let k = 1. - eta * eta * (1. - cosi * cosi);
    if k < 0. {
        Vec3f::new()
    } else {
        *I * eta + n * (eta * cosi - k.sqrt())
    }
}

fn scene_intersect(
    orig: &Vec3f,
    dir: &Vec3f,
    spheres: &[Sphere],
    hit: &mut Vec3f,
    N: &mut Vec3f,
) -> Option<Material> {
    let mut spheres_dist = std::f32::MAX;
    let mut material = Material::new();
    for sphere in spheres.iter() {
        let mut dist_i: &mut f32 = &mut std::f32::MAX;
        if sphere.ray_intersect(&orig, &dir, &mut dist_i) && *dist_i < spheres_dist {
            spheres_dist = *dist_i;
            *hit = (*orig + *dir) * *dist_i;
            *N = (*hit - sphere.center).normalize();
            material = sphere.material;
        }
    }

    if spheres_dist < 1000. {
        Some(material)
    } else {
        None
    }
}

fn cast_ray(orig: &Vec3f, dir: &Vec3f, spheres: &[Sphere], lights: &[Light], depth: u32) -> Vec3f {
    if depth > 4 {
        BACKGROUND
    } else {
        let mut point: Vec3f = Vec3f::new();
        let mut N: Vec3f = Vec3f::new();
        match scene_intersect(&orig, &dir, &spheres, &mut point, &mut N) {
            None => BACKGROUND,
            Some(material) => {
                let mut diffuse_light_intensity = 0.;
                let mut specular_ligth_intensity = 0.;
                let reflect_dir = reflect(&dir, &N).normalize();
                let mut refract_dir = refract(&dir, &N, material.refractive_index);
                refract_dir.normalize();
                let reflect_orig = if reflect_dir * N < 0. {
                    point - N * 1e-3
                } else {
                    point + N * 1e-3
                };
                let refract_orig = if refract_dir * N < 0. {
                    point - N * 1e-3
                } else {
                    point + N * 1e-3
                };
                let reflect_color =
                    cast_ray(&reflect_orig, &reflect_dir, &spheres, &lights, depth + 1);
                let refract_color =
                    cast_ray(&refract_orig, &refract_dir, &spheres, &lights, depth + 1);
                for light in lights.iter() {
                    let light_dir = (light.position - point).normalize();
                    let light_distance = (light.position - point).norm();
                    let ldn = light_dir * N;
                    let shadow_orig = if ldn < 0. {
                        point - N * 1e-3
                    } else {
                        point + N * 1e-3
                    };
                    let mut shadow_pt = Vec3f::new();
                    let mut shadow_N = Vec3f::new();
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

                material.diffuse_color * diffuse_light_intensity * material.albedo.0
                    + Vec3f(1., 1., 1.) * specular_ligth_intensity * material.albedo.1
                    + reflect_color * material.albedo.2
                    + refract_color * material.albedo.3
            }
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
            let dir = Vec3f(x, y, -1.).normalize();
            framebuffer[i + j * WIDTH] = cast_ray(&Vec3f(0., 0., 0.), &dir, spheres, lights, 0);
        }
    }
}

fn main() {
    let mut framebuffer = vec![Vec3f::new(); WIDTH * HEIGHT];
    let ivory = Material {
        albedo: Vec4f(0.6, 0.3, 0.1, 0.),
        diffuse_color: Vec3f(0.4, 0.4, 0.3),
        specular_exponent: 50.,
        refractive_index: 1.,
    };
    let glass = Material {
        albedo: Vec4f(0., 0.5, 0.1, 0.8),
        diffuse_color: Vec3f(0.6, 0.7, 0.8),
        specular_exponent: 125.,
        refractive_index: 1.5,
    };
    let red_rubber = Material {
        albedo: Vec4f(0.9, 0.1, 0., 0.),
        diffuse_color: Vec3f(0.3, 0.1, 0.1),
        specular_exponent: 10.,
        refractive_index: 1.,
    };
    let mirror = Material {
        albedo: Vec4f(0., 10., 0.8, 0.),
        diffuse_color: Vec3f(1., 1., 1.),
        specular_exponent: 1425.,
        refractive_index: 1.,
    };
    let spheres = [
        Sphere {
            center: Vec3f(-3., -0., -16.),
            radius: 2.,
            material: ivory,
        },
        Sphere {
            center: Vec3f(-1.0, -1.5, -12.),
            radius: 2.,
            material: glass,
        },
        Sphere {
            center: Vec3f(1.5, -0.5, -18.),
            radius: 3.,
            material: red_rubber,
        },
        Sphere {
            center: Vec3f(7., 5., -18.),
            radius: 4.,
            material: mirror,
        },
    ];
    let lights = [
        Light {
            position: Vec3f(-20., 20., 20.),
            intensity: 1.5,
        },
        Light {
            position: Vec3f(30., 50., -25.),
            intensity: 1.8,
        },
        Light {
            position: Vec3f(30., 20., 30.),
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
