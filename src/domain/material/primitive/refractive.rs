use rand::prelude::*;
use snafu::prelude::*;

use crate::domain::color::Color;
use crate::domain::material::def::{Material, MaterialKind};
use crate::domain::math::algebra::{Product, Vector};
use crate::domain::math::numeric::{DisRange, Val};
use crate::domain::ray::sampling::{CoefSample, CoefSampling};
use crate::domain::ray::{Ray, RayIntersection, SurfaceSide};
use crate::domain::renderer::Context;

#[derive(Debug, Clone, PartialEq)]
pub struct Refractive {
    color: Color,
    refractive_index: Val,
}

impl Refractive {
    pub fn new(color: Color, refractive_index: Val) -> Result<Self, TryNewRefractiveError> {
        ensure!(refractive_index > Val(0.0), InvalidRefractiveIndexSnafu);

        Ok(Self {
            color,
            refractive_index,
        })
    }

    fn calc_next_reflective_ray(&self, ray: &Ray, intersection: &RayIntersection) -> Ray {
        let normal = intersection.normal();
        let dir = ray.direction();
        Ray::new(
            intersection.position(),
            (dir - Val(2.0) * dir.dot(normal) * normal)
                .normalize()
                .expect("reflective ray's direction should not be zero vector"),
        )
    }

    fn calc_next_refractive_direction(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        cos_i: Val,
        ri: Val,
    ) -> Option<Ray> {
        let normal = intersection.normal();
        let dir_inci = ray.direction();

        let dir_refr_perp = (dir_inci + cos_i * normal) / ri;

        let tmp = Val(1.0) - dir_refr_perp.norm_squared();
        if tmp.is_sign_negative() {
            return None;
        }

        let dir_refr_para = -tmp.sqrt() * normal;

        let dir_refr = (dir_refr_para + dir_refr_perp)
            .normalize()
            .expect("dir_refr should not be zero vector");

        Some(Ray::new(intersection.position(), dir_refr))
    }

    fn calc_reflectance(&self, cos_i: Val, refractive_index: Val) -> Val {
        let r0_sqrt = (Val(1.0) - refractive_index) / (Val(1.0) + refractive_index);
        let r0 = r0_sqrt * r0_sqrt;
        r0 + (Val(1.0) - r0) * (Val(1.0) - cos_i).powi(5)
    }

    fn calc_next_ray(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        reflection_determination: Val,
    ) -> Ray {
        let cos_i = ray.direction().dot(intersection.normal()).abs();
        let ri = if intersection.side() == SurfaceSide::Front {
            self.refractive_index
        } else {
            self.refractive_index.recip()
        };

        let reflectance = self.calc_reflectance(cos_i, ri);
        if reflection_determination < reflectance {
            self.calc_next_reflective_ray(ray, intersection)
        } else if let Some(ray) = self.calc_next_refractive_direction(ray, intersection, cos_i, ri)
        {
            ray
        } else {
            self.calc_next_reflective_ray(ray, intersection)
        }
    }
}

impl Material for Refractive {
    fn material_kind(&self) -> MaterialKind {
        MaterialKind::Refractive
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

impl CoefSampling for Refractive {
    fn coef_sample(
        &self,
        ray: &Ray,
        intersection: &RayIntersection,
        rng: &mut dyn RngCore,
    ) -> CoefSample {
        let reflection_determination = Val(rng.random());
        let direction = self.calc_next_ray(ray, intersection, reflection_determination);
        let pdf = self.coef_pdf(ray, intersection, &direction);
        CoefSample::new(direction, self.color.to_vector(), pdf)
    }

    fn coef_pdf(&self, _ray: &Ray, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        Val(1.0)
    }
}

#[derive(Debug, Snafu, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewRefractiveError {
    #[snafu(display("refractive index is not positive"))]
    InvalidRefractiveIndex,
}

#[cfg(test)]
mod tests {
    use crate::domain::math::algebra::{UnitVector, Vector};
    use crate::domain::math::geometry::Point;

    use super::*;

    #[test]
    fn refractive_new_fails_when_refractive_index_is_invalid() {
        assert!(matches!(
            Refractive::new(Color::WHITE, Val(0.0)),
            Err(TryNewRefractiveError::InvalidRefractiveIndex),
        ));
    }

    #[test]
    fn refractive_calc_next_ray_succeeds_returning_refractive_ray() {
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
            SurfaceSide::Front,
        );

        let refractive = Refractive::new(Color::WHITE, Val(3.0).sqrt()).unwrap();

        let ray_next = refractive.calc_next_ray(&ray, &intersection, Val(1.0));
        assert_eq!(
            ray_next.direction(),
            Vector::new(Val(-0.5), -sqrt3_2, Val(0.0))
                .normalize()
                .unwrap(),
        );
    }

    #[test]
    fn refractive_calc_next_ray_succeeds_when_total_internal_reflection_occurs() {
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

        let refractive = Refractive::new(Color::WHITE, Val(3.0).sqrt()).unwrap();

        let ray_next = refractive.calc_next_ray(&ray, &intersection, Val(1.0));
        assert_eq!(
            ray_next.direction(),
            Vector::new(-sqrt3_2, Val(0.5), Val(0.0))
                .normalize()
                .unwrap(),
        );
    }

    #[test]
    fn refractive_calc_next_ray_succeeds_when_reflectance_is_high() {
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
            SurfaceSide::Front,
        );

        let refractive = Refractive::new(Color::WHITE, Val(3.0).sqrt()).unwrap();

        let ray_next = refractive.calc_next_ray(&ray, &intersection, Val(0.0));
        assert_eq!(
            ray_next.direction(),
            Vector::new(-sqrt3_2, Val(0.5), Val(0.0))
                .normalize()
                .unwrap(),
        );
    }
}
