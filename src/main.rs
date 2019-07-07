#![allow(dead_code)]

extern crate image;
extern crate nalgebra;
extern crate rand;
extern crate rayon;

mod camera;
mod hitable;
mod materials;
mod ray;
mod rectangle;
mod sphere;
mod texture;
mod world;

use std::env;
use std::f32;

use nalgebra::core::Vector3;
use rand::thread_rng;
use rayon::prelude::*;

use camera::Camera;
use materials::{Diffuse, Light, Reflective, Refractive};
use sphere::Sphere;
use rectangle::Rectangle;
use texture::{ConstantTexture, ImageTexture};
use world::World;


fn random_scene() -> World {
    let mut world = World::new();

    world.add(Sphere::new(
        Vector3::new(0.0, -1000.0, 0.0),
        1000.0,
        Diffuse::new(ConstantTexture::new(0.5, 0.5, 0.5)))
        );

    for a in -11..11 {
        for b in -11..11 {
            let material = rand::random::<f32>();
            let center: Vector3<f32> = Vector3::new(a as f32 + 0.9 * rand::random::<f32>(),
                                                    0.2,
                                                    b as f32 + 0.9 * rand::random::<f32>());

            if (center - Vector3::new(4.0, 0.2, 0.0)).norm() > 0.9 {
                if material < 0.75 {
                    world.add(Sphere::new(center, 0.2, Diffuse::new(
                                ConstantTexture::new(
                                    rand::random::<f32>() * rand::random::<f32>(),
                                    rand::random::<f32>() * rand::random::<f32>(),
                                    rand::random::<f32>() * rand::random::<f32>()))));
                }
                else if material < 0.95 {
                    world.add(Sphere::new(center, 0.2, Reflective::new(
                                Vector3::new(
                                    0.5 * (1.0 * rand::random::<f32>()),
                                    0.5 * (1.0 * rand::random::<f32>()),
                                    0.5 * (1.0 * rand::random::<f32>())),
                                    0.5 * rand::random::<f32>())));
                }
                else {
                    world.add(Sphere::new(center, 0.2, Refractive::new(
                                Vector3::new(0.9, 0.9, 0.9), 1.5, 0.0)));

                    world.add(Sphere::new(center, -0.19, Refractive::new(
                                Vector3::new(0.9, 0.9, 0.9), 1.5, 0.0)));
                }
            }
        }
    }

    world.add(Sphere::new(
        Vector3::new(-2.0, 1.0, 0.0),
        1.0,
        Diffuse::new(ConstantTexture::new(0.75, 0.25, 0.25)))
        );

    world.add(Sphere::new(
        Vector3::new(0.0, 1.0, 0.0),
        1.0,
        Refractive::new(Vector3::new(1.0, 1.0, 1.0), 1.5, 0.0))
        );

    world.add(Sphere::new(
        Vector3::new(0.0, 1.0, 0.0),
        -0.99,
        Refractive::new(Vector3::new(1.0, 1.0, 1.0), 1.5, 0.0))
        );

    world.add(Sphere::new(
        Vector3::new(2.0, 1.0, 0.0),
        1.0,
        Reflective::new(Vector3::new(0.5, 0.5, 0.5), 0.05))
        );

    world
}


fn earth() -> World {
    let mut world = World::new();

    world.add(Sphere::new(
            Vector3::new(0.0, 0.0, 0.0),
            2.0,
            Diffuse::new(ImageTexture::new("earthmap.png"))
            ));

    world

}


fn simple_light() -> World {
    let mut world = World::new();

    world.add(Sphere::new(
        Vector3::new(0.0, -1000.0, 0.0),
        1000.0,
        Diffuse::new(ConstantTexture::new(0.5, 0.5, 0.5)))
        );

    world.add(Sphere::new(
            Vector3::new(0.0, 2.0, 0.0),
            2.0,
            Diffuse::new(ConstantTexture::new(1.0, 0.0, 0.0))
            ));

    world.add(Sphere::new(
            Vector3::new(0.0, 7.0, 0.0),
            2.0,
            Light::new(ConstantTexture::new(4.0, 4.0, 4.0))
            ));

    world.add(Rectangle::new(3.0, 5.0, 1.0, 3.0, -2.0,
                             Light::new(ConstantTexture::new(4.0, 4.0, 4.0))));

    world
}


fn main() {
    let (width, height): (u32, u32) = (1920, 960);
    let args: Vec<String> = env::args().collect();
    let samples: u32 = args[1].parse().unwrap();

    let origin = Vector3::new(13.0, 3.0, 3.0);
    let lookat = Vector3::new(0.0, 0.0, 0.0);
    let camera = Camera::new(origin,
                             lookat,
                             Vector3::new(0.0, 1.0, 0.0),
                             50.0,
                             (width / height) as f32,
                             0.0,
                             10.0);

    let world = simple_light();

    let mut pixels = vec![image::Rgb([0, 0, 0]); (width * height) as usize];
    pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
        let mut coordinate: Vector3<f32> = Vector3::zeros();
        let x = i % width as usize;
        let y = i / width as usize;

        let mut rng = thread_rng();

        (0..samples).for_each(|_| {
            let u = (x as f32 + rand::random::<f32>()) / width as f32;
            let v = (y as f32 + rand::random::<f32>()) / height as f32;
            let ray = camera.get_ray(u, v);
            coordinate += ray::compute_color(&ray, &world, 0, &mut rng);
        });

        coordinate /= samples as f32;
        (0..3).for_each(|i| coordinate[i] = 255.0 * coordinate[i].sqrt());
        *pixel = image::Rgb([coordinate.x as u8, coordinate.y as u8, coordinate.z as u8]);
    });


    let mut buffer = image::ImageBuffer::new(width, height);

    buffer.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
        let index = (y * width + x) as usize;
        *pixel = pixels[index];
    });

    image::ImageRgb8(buffer).flipv().save("render.png").unwrap();
}
