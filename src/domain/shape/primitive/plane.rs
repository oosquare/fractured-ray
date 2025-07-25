use std::ops::RangeBounds;

use crate::domain::math::algebra::{Product, UnitVector};
use crate::domain::math::geometry::Point;
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::sampling::LightSampling;
use crate::domain::ray::{Ray, RayIntersection, SurfaceSide};
use crate::domain::shape::def::{BoundingBox, Shape, ShapeId, ShapeKind};

#[derive(Debug, Clone, PartialEq)]
pub struct Plane {
    point: Point,
    normal: UnitVector,
}

impl Plane {
    pub fn new(point: Point, normal: UnitVector) -> Self {
        Self { point, normal }
    }

    pub fn point(&self) -> Point {
        self.point
    }

    pub fn normal(&self) -> UnitVector {
        self.normal
    }

    pub fn calc_ray_intersection(
        ray: &Ray,
        range: DisRange,
        point: &Point,
        normal: &UnitVector,
    ) -> Option<RayIntersection> {
        let den = ray.direction().dot(*normal);
        if den != Val(0.0) {
            let num = (*point - ray.start()).dot(*normal);
            let distance = num / den;
            if distance > Val(0.0) && range.contains(&distance) {
                let position = ray.at(distance);
                let (normal, side) = if den < Val(0.0) {
                    (*normal, SurfaceSide::Front)
                } else {
                    (-(*normal), SurfaceSide::Back)
                };
                Some(RayIntersection::new(distance, position, normal, side))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Shape for Plane {
    fn shape_kind(&self) -> ShapeKind {
        ShapeKind::Plane
    }

    fn hit(&self, ray: &Ray, range: DisRange) -> Option<RayIntersection> {
        Self::calc_ray_intersection(ray, range, &self.point, &self.normal)
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        None
    }

    fn get_sampler(&self, _shape_id: ShapeId) -> Option<Box<dyn LightSampling>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::Vector;

    use super::*;

    #[test]
    fn plane_hit_succeeds() {
        let plane = Plane::new(
            Point::new(Val(-1.0), Val(0.0), Val(0.0)),
            UnitVector::x_direction(),
        );
        let ray = Ray::new(
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            Vector::new(Val(-1.0), Val(0.0), Val(-1.0))
                .normalize()
                .unwrap(),
        );
        let intersection = plane.hit(&ray, DisRange::positive()).unwrap();
        assert_eq!(intersection.distance(), Val(2.0).sqrt());
        assert_eq!(
            intersection.position(),
            Point::new(Val(-1.0), Val(0.0), Val(-1.0))
        );
        assert_eq!(intersection.normal(), UnitVector::x_direction());
        assert_eq!(intersection.side(), SurfaceSide::Front);
    }
}
