use super::{
    *, 
    controller_calibration_plugin::{
        ControllerCalibrationPlugin, LeftAxisWarp
    }, 
    distance_field_plugin::{
        DistanceFieldPlugin, DistanceFieldObstacle, DistanceField
    }
};
use bevy_rapier2d::prelude::*;

/*
    COMPONENTS
*/

#[derive(Component)]
pub struct Player{}

#[derive(Component)]
pub struct AimReticle{}

#[derive(Component)]
pub struct RopeAnchor{}

/*
    PLUGIN
*/

pub struct GrapplePlugin;
impl Plugin for GrapplePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0),
                RapierDebugRenderPlugin::default(),
                ControllerCalibrationPlugin{},
                DistanceFieldPlugin{}
            ))
            .add_systems(Startup, setup_player)
            .add_systems(Update, (
                rotate_reticle,
                shorten_joint_length
            ));
    }
}

/*
    STARTUP SYSTEMS
*/

fn setup_player(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>
) {
    //Set Gravity
    rapier_config.gravity = Vec2::NEG_Y*9.81;

    //setup player and anchor
    let player_position = Vec3::new(0.0, 50.0, 0.0);
    let anchor_position = Vec3::new(150.0, 100.0, 0.0);
    let anchor_size = Vec3::new(200.0, 200.0, 0.0);
    //spawn Player
    let player = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(player_position),
            ..Default::default()
        },
        RigidBody::Dynamic,
        Velocity::zero(),
        Collider::cuboid(5.0, 5.0),
        Friction{coefficient: 0.0, combine_rule: CoefficientCombineRule::Average},
        Restitution{coefficient: 0.0, combine_rule: CoefficientCombineRule::Average},
        GravityScale(1.0),
        LockedAxes::ROTATION_LOCKED,
        AdditionalMassProperties::MassProperties(MassProperties { local_center_of_mass: Vec2::ZERO, mass: 1f32, principal_inertia: 5.0*5.0*2.0/3.0 }),
        Player{}
    )).id();
    //Spawn Block with Grapple Anchor
    let anchor = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 1.0, 0.0),
                custom_size: Some(anchor_size.truncate()),
                ..Default::default()
            },
            transform: Transform::from_translation(anchor_position),
            ..Default::default()
        },
        RigidBody::Fixed,
        Collider::cuboid(100.0, 100.0),
        RopeAnchor{},
        DistanceFieldObstacle{}
    )).id();
    //Create Joint
    let joint_length: f32 = (anchor_position-anchor_size/2.0 - player_position).length();
    let joint = RopeJointBuilder::new()
        .local_anchor2(anchor_size.truncate()/-2.0)
        .limits([0.0, (joint_length.powi(2)/2f32).sqrt()]);
    //Add Joint
    commands.entity(anchor).insert(ImpulseJoint::new(player, joint));

    //Create Player Aiming Reticle
    let reticle = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(4.0, 4.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
            ..Default::default()
        },
        AimReticle{}
    )).id();
    //Add Aim reticle as a child of player
    commands.entity(player).push_children(&[reticle]);

    //spawn ground
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.6, 1.0, 0.7),
                custom_size: Some(Vec2::new(400.0, 200.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3{x: 50.0, y: -140.0, z: 1.0}),
            ..Default::default()
        },
        Collider::cuboid(200.0, 100.0),
        DistanceFieldObstacle{}
    ));
}

/*
    UPDATE SYSTEMS
*/

fn rotate_reticle(
    gamepads: Res<Gamepads>, 
    input: Res<Axis<GamepadAxis>>,
    warp: Res<LeftAxisWarp>,
    mut query: Query<&mut Transform, With<AimReticle>>
){
    if let Some(gamepad) = gamepads.iter().next(){
        let axis_lx = GamepadAxis{gamepad: gamepad, axis_type: GamepadAxisType::LeftStickX};
        let axis_ly = GamepadAxis{gamepad: gamepad, axis_type: GamepadAxisType::LeftStickY};
        if let Some(y) = input.get(axis_ly) {
            if let Some(x) = input.get(axis_lx){
                //let angle: f32 = (y*2.-1.).atan2(x*2.-1.);
                for mut transform in query.iter_mut(){
                    transform.translation = Vec3::from(((warp.warp*Vec2{x,y}).clamp_length_max(1.0), 3.0))*5.0;
                }
            }
        }
    }
}

fn shorten_joint_length(
    player: Query<(&GlobalTransform, &Velocity), (With<Player>, Without<RopeAnchor>)>,
    mut anchor: Query<(&GlobalTransform, &mut ImpulseJoint), (With<RopeAnchor>, Without<Player>)>,
    field: Res<DistanceField>,
    time: Res<Time>
){
    /*let (pos2, mut joint) = anchor.single_mut();
    let (pos1, vel) = player.single();
    let cur_dist = (pos1.translation().truncate()+joint.data.local_anchor1())
        .distance(pos2.translation().truncate() + joint.data.local_anchor2());
    if (joint.data.limits(bevy_rapier2d::rapier::prelude::JointAxis::X).unwrap().max.powi(2)*2f32).sqrt() > cur_dist{
        joint.data.set_limits(bevy_rapier2d::rapier::prelude::JointAxis::X, [0f32, (cur_dist*cur_dist/2f32).sqrt()]);
        joint.data.set_limits(bevy_rapier2d::rapier::prelude::JointAxis::Y, [0f32, (cur_dist*cur_dist/2f32).sqrt()]);
    }
    if field.distance_map.len() == 0{
        return;
    }
    let pos = (pos1.translation().truncate() - field.center + field.half_extents)/
        (field.half_extents*2.0)*
        Vec2::new(field.sample_dimensions.0 as f32, field.sample_dimensions.1 as f32);
    let index = pos.floor();
    let lerp_coeff = pos.fract();
    if index.min(Vec2::ZERO)==Vec2::ZERO && 
        index.min(Vec2::new(field.sample_dimensions.0 as f32-1.0, field.sample_dimensions.1 as f32-1.0))==index
    {
        let least_change = distance_field_plugin::lerp(
            field.least_change_map[index.x as usize][index.y as usize + 1],
            field.least_change_map[index.x as usize + 1][index.y as usize + 1],
            field.least_change_map[index.x as usize][index.y as usize],
            field.least_change_map[index.x as usize + 1][index.y as usize],
            lerp_coeff.x,
            lerp_coeff.y
        ).normalize();
        let gradient = -distance_field_plugin::lerp(
            field.gradient_map[index.x as usize][index.y as usize + 1],
            field.gradient_map[index.x as usize + 1][index.y as usize + 1],
            field.gradient_map[index.x as usize][index.y as usize],
            field.gradient_map[index.x as usize + 1][index.y as usize],
            lerp_coeff.x,
            lerp_coeff.y
        ).normalize();
        let dist = distance_field_plugin::lerp(
            field.interference_map[index.x as usize][index.y as usize + 1],
            field.interference_map[index.x as usize + 1][index.y as usize + 1],
            field.interference_map[index.x as usize][index.y as usize],
            field.interference_map[index.x as usize + 1][index.y as usize],
            lerp_coeff.x,
            lerp_coeff.y
        );
        let dir1 = least_change + gradient*dist*dist*0.5;
        let dir2 = -least_change + gradient*dist*dist*0.5;
        let to_player = (pos1.translation()-pos2.translation()).truncate();
        let limit_delta: f32;
        if dir1.normalize().dot(vel.linvel) > dir2.normalize().dot(vel.linvel){
            limit_delta = dir1.dot(to_player)/to_player.length()*time.delta_seconds()*30.0;
        }else{
            limit_delta = dir2.dot(to_player)/to_player.length()*time.delta_seconds()*30.0;
        }
        let new_dist = cur_dist + limit_delta;
        joint.data.set_limits(bevy_rapier2d::rapier::prelude::JointAxis::X, [0f32, (new_dist*new_dist/2f32).sqrt()]);
        joint.data.set_limits(bevy_rapier2d::rapier::prelude::JointAxis::Y, [0f32, (new_dist*new_dist/2f32).sqrt()]);
    }*/
}
