use rand::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::algebra::{Product, Vector};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::sampling::{CoefSample, CoefSampling};
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::renderer::Context;

#[derive(Debug, Clone, PartialEq)]
pub struct Specular {
    color: Color,
}

impl Specular {
    pub fn new(albedo: Color) -> Self {
        Self { color: albedo }
    }

    fn calc_next_ray(&self, ray: &Ray, intersection: &RayIntersection) -> Ray {
        let normal = intersection.normal();
        let dir = ray.direction();
        Ray::new(
            intersection.position(),
            (dir - Val(2.0) * dir.dot(normal) * normal)
                .normalize()
                .expect("reflective ray's direction should not be zero vector"),
        )
    }
}

impl Material for Specular {
    fn material_kind(&self) -> MaterialKind {
        MaterialKind::Specular
    }

    fn bsdf(&self, _ray: &Ray, _intersection: &RayIntersection, _ray_next: &Ray) -> Vector {
        unimplemented!("dirac function in refractive BSDF can't be represented")
    }

    fn shade(
        &self,
        context: &mut Context<'_>,
        ray: Ray,
        intersection: RayIntersection,
        depth: usize,
    ) -> Color {
        let sample = self.coef_sample(&ray, &intersection, *context.rng());
        let coefficient = sample.coefficient();
        let ray_next = sample.into_ray();

        let renderer = context.renderer();
        let radiance = renderer.trace(context, ray_next, DisRange::positive(), depth + 1);
        coefficient * radiance
    }

    fn as_dyn(&self) -> &dyn Material {
        self
    }
}

impl CoefSampling for Specular {
    fn coef_sample(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        _rng: &mut dyn RngCore,
    ) -> CoefSample {
        let direction = self.calc_next_ray(ray, intersection);
        let pdf = self.coef_pdf(ray, intersection, &direction);
        CoefSample::new(direction, self.color.to_vector(), pdf)
    }

    fn coef_pdf(&self, _ray: &Ray, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        Val(1.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::{UnitVector, Vector};
    use crate::domain::math::geometry::Point;
    use crate::domain::ray::SurfaceSide;

    use super::*;

    #[test]
    fn specular_calc_next_ray_succeeds() {
        let sqrt3_2 = Val(3.0).sqrt() / Val(2.0);

        let ray = Ray::new(
            Point::new(sqrt3_2, Val(0.5), Val(0.0)),
            Vector::new(-sqrt3_2, Val(-0.5), Val(0.0))
                .normalize()
                .unwrap(),
        );

        let intersection = RayIntersection::new(
            Val(1.0),
            Point::new(Val(0.0), Val(0.0), Val(0.0)),
            UnitVector::y_direction(),
            SurfaceSide::Back,
        );

        let specular = Specular::new(Color::WHITE);

        let ray_next = specular.calc_next_ray(&ray, &intersection);
        assert_eq!(
            ray_next.direction(),
            Vector::new(-sqrt3_2, Val(0.5), Val(0.0))
                .normalize()
                .unwrap(),
        );
    }
}
