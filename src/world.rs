use std::sync::Arc;

use nalgebra::core::Vector3;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use materials::Diffuse;
use ray::Ray;
use texture::ConstantTexture;

#[derive(Clone)]
/// The World struct holds all of the objects in the scene
pub struct World {
    pub objects: Vec<Arc<dyn Hitable>>,
}

impl World {
    /// Create a new World to hold all of the objects in the scene
    pub fn new() -> World {
        World { objects: Vec::new() }
    }

    /// Add objects to the instantiated world
    ///
    /// We use a 'static lifetime so that we can Arc
    /// object inside the function rather than having to
    /// pass object as an Arced object as an input parameter.
    pub fn add<H: Hitable + 'static>(&mut self, object: H) {
        let object = Arc::new(object);
        self.objects.push(object);
    }
}

impl Hitable for World {
    /// Determine if the given ray has hit any of the objects in the world
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        let mut record = HitRecord::new(0.0,
                                        0.0,
                                        0.0,
                                        Vector3::zeros(),
                                        Vector3::zeros(),
                                        Arc::new(Diffuse::new(ConstantTexture::new(0.0, 0.0,
                                                                                   0.0))));
        let mut hit_anything: bool = false;
        let mut closed_so_far: f32 = position_max;

        for object in &self.objects {
            match object.hit(ray, position_min, closed_so_far) {
                None => (),
                Some(hit_record) => {
                    hit_anything = true;
                    closed_so_far = hit_record.parameter;
                    record = hit_record;
                }
            }
        }

        return if hit_anything { Some(record) } else { None };
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if self.objects.len() > 0 {
            if let Some(accumulated_box) = self.objects.first().unwrap().bounding_box(t0, t1) {
                for i in 1..self.objects.len() {
                    if let Some(new_box) = self.objects[i].bounding_box(t0, t1) {
                        return Some(accumulated_box.surrounding_box(&new_box))
                    } else {
                        return None
                    }
                }
                return None;
            } else {
                return None;
            }
        }
        None
    }
}
