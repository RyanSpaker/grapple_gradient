use bevy::prelude::*;
use super::simulation::UpdateRungeKutta;


pub struct FlatlandPhysicsPlugin{}
impl Plugin for FlatlandPhysicsPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            UpdateRungeKutta,
            UpdateRungeKutta,
            UpdateRungeKutta
        ));
    }
}