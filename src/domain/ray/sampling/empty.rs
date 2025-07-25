use rand::prelude::*;

use crate::domain::material::def::Material;
use crate::domain::math::numeric::Val;
use crate::domain::ray::{Ray, RayIntersection};
use crate::domain::shape::def::{Shape, ShapeId};

use super::{LightSample, LightSampling};

#[derive(Debug, Clone, PartialEq)]
pub struct EmptySampler {}

impl EmptySampler {
    pub fn new() -> Self {
        Self {}
    }
}

impl LightSampling for EmptySampler {
    fn id(&self) -> Option<ShapeId> {
        None
    }

    fn shape(&self) -> Option<&dyn Shape> {
        None
    }

    fn light_sample(
        &self,
        _ray: &Ray,
        _intersection: &RayIntersection,
        _material: &dyn Material,
        _rng: &mut dyn RngCore,
    ) -> Option<LightSample> {
        None
    }

    fn light_pdf(&self, _intersection: &RayIntersection, _ray_next: &Ray) -> Val {
        Val(0.0)
    }
}
