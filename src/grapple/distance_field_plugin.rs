use std::{cmp::Ordering, f32::consts::{PI, TAU}, ops::{Add, Mul, Sub}};
use futures_lite::future;
use bevy::{tasks::{AsyncComputeTaskPool, Task}, render::{render_resource::{Extent3d, TextureDimension, TextureFormat}, texture::ImageSampler}, window::PrimaryWindow, ecs::world};
use bevy_rapier2d::prelude::Collider;
use super::*;

/*
    COMPONENTS
*/

#[derive(Component)]
pub struct DistanceFieldObstacle{}

#[derive(Component)]
pub struct DistanceFieldComputeTask(Task<DistanceField>);

/*
    RESOURCES
*/

#[derive(Resource)]
pub struct DistanceField{
    pub center: Vec2,
    pub half_extents: Vec2,
    pub sample_dimensions: (usize, usize),
    pub distance_field: Vec<Vec<f32>>,
    pub gradient_field: Vec<Vec<Vec2>>,
    pub curl_field: Vec<Vec<f32>>
}
impl Default for DistanceField{
    fn default() -> Self {
        Self{
            center: Vec2::ZERO, 
            half_extents: Vec2::new(800.0, 400.0), 
            sample_dimensions: (2000, 2000), 
            distance_field: vec![],
            gradient_field: vec![],
            curl_field: vec![]
        }
    }
}

/*
    PLUGIN
*/

pub struct DistanceFieldPlugin{}
impl Plugin for DistanceFieldPlugin{
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DistanceField>()
            .add_systems(Startup, (debug_setup_image, debug_setup_mouse_pointers))
            .add_systems(Update, (
                spawn_compute_fields_task.run_if(should_update_distance_field),
                handle_compute_fields_task,
                debug_update_image,
                debug_update_mouse_pointers
            ));
    }
}

/*
    UPDATE SYSTEMS
*/

fn should_update_distance_field(query: Query<Entity, Added<Collider>>) -> bool{
    return !query.is_empty();
}
fn spawn_compute_fields_task(
    mut commands: Commands,
    field: Res<DistanceField>, 
    colliders: Query<(&Transform, &Collider), With<DistanceFieldObstacle>>
){
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(calculate_fields(
        field.center, 
        field.half_extents, 
        field.sample_dimensions, 
        colliders.iter().map(|a|(a.1.clone(), *a.0) ).collect()
    ));
    commands.spawn(DistanceFieldComputeTask(task));
}
async fn calculate_fields(
    center: Vec2, 
    half_extents: Vec2, 
    sample_counts: (usize, usize), 
    colliders: Vec<(Collider, Transform)>
) -> DistanceField{
    //setup variables
    let origin = center - half_extents;
    let step = half_extents*2.0/Vec2::new(sample_counts.0 as f32-1.0, sample_counts.1 as f32-1.0);
    //calculate distance field:
    let mut dist_field: Vec<Vec<f32>> = vec![vec![0.0; sample_counts.1]; sample_counts.0];
    let colliders: Vec<(&Collider, Vec2, f32)> = colliders.iter()
        .map(|(col, trans)| (col, trans.translation.truncate(), trans.rotation.to_euler(EulerRot::ZXY).0))
        .collect();
    for x in 0..sample_counts.0{
        for y in 0..sample_counts.1{
            let center = origin + step*Vec2::new(x as f32, y as f32);
            let mut dists = colliders.iter().map(|(col, trans, rot)| {
                col.distance_to_point(
                    trans.to_owned(), 
                    rot.to_owned(),
                    center, 
                    true
                )
            }).collect::<Vec<f32>>();
            
            dists.sort_by(|a, b| a.partial_cmp(&b).unwrap_or(Ordering::Equal));
            dist_field[x][y] = dists[0];
        }
    }
    let mut gradient: Vec<Vec<Vec2>> = vec![vec![Vec2::ZERO; sample_counts.1]; sample_counts.0];
    for x in 1..(sample_counts.0-1){
        for y in 1..(sample_counts.1-1){
            // Sobel Kernel for discrete gradient
            gradient[x][y] = Vec2::new(
                dist_field[x+1][y]*2.0 + dist_field[x+1][y+1] + dist_field[x+1][y-1]
                -dist_field[x-1][y]*2.0 - dist_field[x-1][y-1] - dist_field[x-1][y+1],
                dist_field[x][y+1]*2.0 + dist_field[x+1][y+1] + dist_field[x-1][y+1]
                -dist_field[x][y-1]*2.0 - dist_field[x+1][y-1] - dist_field[x-1][y-1]
            );
            gradient[x][y] /= step;
        }
    }
    let mut curl: Vec<Vec<f32>> = vec![vec![0.0; sample_counts.1]; sample_counts.0];
    for x in 2..(sample_counts.0-2){
        for y in 2..(sample_counts.1-2){
            let grad_bl = gradient[x-1][y-1];
            let grad_br = gradient[x+1][y-1];
            let grad_b = gradient[x][y-1];
            let grad_l = gradient[x-1][y];
            let grad_r = gradient[x+1][y];
            let grad_t = gradient[x][y+1];
            let grad_tl = gradient[x-1][y+1];
            let grad_tr = gradient[x+1][y+1];
            let cw_grad_bl = Vec2::new(grad_bl.y, grad_bl.x*-1.0).normalize();
            let cw_grad_br = Vec2::new(grad_br.y, grad_br.x*-1.0).normalize();
            let cw_grad_b = Vec2::new(grad_b.y, grad_b.x*-1.0).normalize();
            let cw_grad_l = Vec2::new(grad_l.y, grad_l.x*-1.0).normalize();
            let cw_grad_r = Vec2::new(grad_r.y, grad_r.x*-1.0).normalize();
            let cw_grad_t = Vec2::new(grad_t.y, grad_t.x*-1.0).normalize();
            let cw_grad_tl = Vec2::new(grad_tl.y, grad_tl.x*-1.0).normalize();
            let cw_grad_tr = Vec2::new(grad_tr.y, grad_tr.x*-1.0).normalize();
            let dy_dx = cw_grad_r.y*2.0 + cw_grad_tr.y + cw_grad_br.y - cw_grad_l.y*2.0 - cw_grad_bl.y - cw_grad_tl.y;
            let dx_dy = cw_grad_t.x*2.0 + cw_grad_tr.x + cw_grad_tl.x - cw_grad_b.x*2.0 - cw_grad_bl.x - cw_grad_br.x;
            curl[x][y] = dy_dx - dx_dy;
        }
    }
    //finished
    return DistanceField{ 
        center, 
        half_extents, 
        sample_dimensions: sample_counts, 
        distance_field: dist_field,
        gradient_field: gradient,
        curl_field: curl
    };
}

fn sample_grid<T>(position: Vec2, field: &Vec<Vec<T>>) -> T
where T: std::ops::Mul<f32, Output = T> + std::ops::Add<T, Output = T> + Clone
{
    let index = position;
    let x = index.x as usize;
    let y = index.y as usize;
    let p_x = index.x%1.0;
    let p_y = index.y%1.0;
    let bottom = field[x][y].clone()*(1.0-p_x) + field[x+1][y].clone()*p_x;
    let top = field[x][y+1].clone()*(1.0-p_x) + field[x+1][y+1].clone()*p_x;
    return bottom*(1.0-p_y) + top*p_y;
}
fn sample_grid_checked<T>(position: Vec2, field: &Vec<Vec<T>>) -> Option<T>
where T: std::ops::Mul<f32, Output = T> + std::ops::Add<T, Output = T> + Clone
{
    let index = position;
    let x = index.x as usize;
    let y = index.y as usize;
    let p_x = index.x%1.0;
    let p_y = index.y%1.0;
    if x < 0 || x >= field.len()-1 || y < 0 || y >= field[0].len()-1{ return None;}
    let bottom = field[x][y].clone()*(1.0-p_x) + field[x+1][y].clone()*p_x;
    let top = field[x][y+1].clone()*(1.0-p_x) + field[x+1][y+1].clone()*p_x;
    return Some(bottom*(1.0-p_y) + top*p_y);
}

fn handle_compute_fields_task(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut DistanceFieldComputeTask)>,
    mut fields: ResMut<DistanceField>
) {
    for (entity, mut task) in &mut tasks {
        if let Some(new_field) = future::block_on(future::poll_once(&mut task.0)) {
            *fields = new_field;
            commands.entity(entity).despawn();
        }
    }
}

/*
    DEBUG CODE
*/

#[derive(Component)]
pub struct FieldImage{}
fn debug_setup_image(
    field: Res<DistanceField>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>
){
    let handle = images.add(Image::new_fill(
        Extent3d{width: field.sample_dimensions.0 as u32, height: field.sample_dimensions.1 as u32, depth_or_array_layers: 1},
        TextureDimension::D2, 
        [255f32, 255f32, 255f32, 255f32].into_iter().flat_map(|f| f.to_le_bytes().into_iter()).collect::<Vec<u8>>().as_slice(),
        TextureFormat::Rgba32Float
    ));
    images.get_mut(&handle).unwrap().sampler_descriptor = ImageSampler::nearest();
    commands.spawn((
        SpriteBundle{
            sprite: Sprite {
                color: Color::rgb(1.0, 1.0, 1.0),
                custom_size: Some(field.half_extents*2.0),
                ..Default::default()
            },
            transform: Transform::from_translation(field.center.extend(0.0)),
            texture: handle,
            ..Default::default()
        },
        FieldImage{}
    ));
}
fn debug_update_image(
    mut image: Query<(&Handle<Image>, &mut Sprite, &mut Transform), With<FieldImage>>,
    mut images: ResMut<Assets<Image>>,
    field: Res<DistanceField>
){
    if !field.is_changed() || field.distance_field.len() == 0{
        return;
    }
    for (img, mut sprite, mut trans) in image.iter_mut(){
        if let Some(data) = images.get_mut(&img){
            let choice = 3;
            match choice{
                0 => {
                    /*data.data.clone_from(&(0..field.sample_dimensions.1).into_iter().rev().map(|y| {
                        (0..field.sample_dimensions.0).into_iter().map(|x| {
                            Color::Hsla { hue: field.interference_map[x][y]*1800.0, saturation: 1.0, lightness: 0.5, alpha: 0.9 }
                                .as_rgba_f32().into_iter().flat_map(|f| f.to_le_bytes().into_iter())
                        }).flatten().collect::<Vec<u8>>()
                    }).flatten().collect::<Vec<u8>>());*/
                },
                1 => {
                    data.data.clone_from(&(0..field.sample_dimensions.1).into_iter().rev().map(|y| {
                        (0..field.sample_dimensions.0).into_iter().map(|x| {
                            Color::Hsla { hue: field.distance_field[x][y]*2.0%360.0, saturation: 1.0, lightness: 0.5, alpha: 1.0 }
                                .as_rgba_f32().into_iter().flat_map(|f| f.to_le_bytes().into_iter())
                        }).flatten().collect::<Vec<u8>>()
                    }).flatten().collect::<Vec<u8>>());
                },
                2 => {
                    data.data.clone_from(&(0..field.sample_dimensions.1).into_iter().rev().map(|y| {
                        (0..field.sample_dimensions.0).into_iter().map(|x| {
                            Color::Hsla { hue: field.gradient_field[x][y].angle_between(Vec2::X).rem_euclid(TAU) / TAU * 360.0, saturation: 1.0, lightness: 0.5, alpha: 1.0 }
                                .as_rgba_f32().into_iter().flat_map(|f| f.to_le_bytes().into_iter())
                        }).flatten().collect::<Vec<u8>>()
                    }).flatten().collect::<Vec<u8>>());
                },
                3 => {
                    data.data.clone_from(&(0..field.sample_dimensions.1).into_iter().rev().map(|y| {
                        (0..field.sample_dimensions.0).into_iter().map(|x| {
                            Color::rgb(field.curl_field[x][y], field.curl_field[x][y]*-1.0, 0.0)
                                .as_rgba_f32().into_iter().flat_map(|f| f.to_le_bytes().into_iter())
                        }).flatten().collect::<Vec<u8>>()
                    }).flatten().collect::<Vec<u8>>());
                },
                _ => {}
            }
        }
        sprite.custom_size = Some(field.half_extents*2.0);
        trans.translation = field.center.extend(3.0);
    }
}

#[derive(Component)]
pub struct DebugPointer{}
fn debug_setup_mouse_pointers(mut commands: Commands){
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 0.0, 1.0),
                custom_size: Some(Vec2::new(10.0, 1.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3{x: -5.0, y: 0.0, z: 8.0}),
            ..Default::default()
        },
        DebugPointer{}
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(10.0, 1.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3{x: 5.0, y: 0.0, z: 10.0}),
            ..Default::default()
        },
        DebugPointer{}
    ));
}
fn debug_update_mouse_pointers(
    field: Res<DistanceField>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut pointer: Query<&mut Transform, With<DebugPointer>>
){
    if field.distance_field.len() == 0{
        return;
    }
    let (camera, camera_transform) = camera_q.single();
    if let Some(world_position) = window.single().cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
    }
}

pub fn lerp<T, U>(a: T, b: T, c: T, d: T, x: U, y: U) -> T where 
    T: Mul<U, Output = T> + Add<T, Output = T> + Copy,
    U: Mul<U, Output = U> + Add<U> + Sub<U, Output = U> + Copy
{
    a*(x-x*y) + b + b*(x*y-x-y) + c*(x*y) + d*(y-x*y) 
}

