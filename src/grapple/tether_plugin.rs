use bevy_rapier2d::prelude::RopeJoint;
use super::*;

#[derive(Component)]
pub struct Tether2D{
    joint: RopeJoint,
    auto_retract: bool,
    elastic: bool,
    enabled: bool
}
impl Default for Tether2D{
    fn default() -> Self {
        Self { joint: RopeJoint::default(), auto_retract: true, elastic: false, enabled: false }
    }
}
impl Tether2D{
    pub fn set_anchors(&mut self, local_anchor_a: Vec2, local_anchor_b: Vec2){
        self.joint.set_local_anchor1(local_anchor_a);
        self.joint.set_local_anchor2(local_anchor_b);
        
    }
    pub fn set_distance(&mut self, distance: f32){
        self.joint.set_limits([0f32, distance]);
    }
}