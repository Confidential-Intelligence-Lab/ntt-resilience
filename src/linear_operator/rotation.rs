#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rotation(pub isize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotationPlan {
    rotations: Vec<Rotation>,
}

impl RotationPlan {
    pub fn new(rotations: Vec<Rotation>) -> Self {
        Self { rotations }
    }

    pub fn rotations(&self) -> &[Rotation] {
        &self.rotations
    }
}
