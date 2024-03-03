mod colliders;

use bevy::{prelude::*, utils::{HashSet, hashbrown::HashMap}};
use nalgebra::Matrix3;
pub use colliders::*;

#[derive(Component)]
pub struct RigidBody2D{
    pub mass: f32,
    pub rotational_inertia: f32
}

#[derive(Component)]
pub struct RotationalInertiaTensorList2D{
    pub inertia: HashMap<u32, f32>
}