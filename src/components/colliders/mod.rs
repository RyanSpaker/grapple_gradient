use bevy::prelude::*;

#[derive(Component)]
pub struct BoxCollider2D{
    pub center_position: Vec2,
    pub half_extents: Vec2,
    pub restitution: f32,
    pub friction: f32
}