use bevy::prelude::*;
use super::components::RigidBody2D;

//Contains all of the config values for the differential equation solver which uses the Runge Kutta Method
#[derive(Resource)]
pub struct DiffEqSolverConfig{
    //Number of intermediate steps in the Runge Kutta Method
    order: usize,
    //Vector of length <order> holding node constants for Runge-Kutta Method, commonly notated as (c)
    nodes: Vec<f32>, 
    //Vector of length <order> holding weight constants for Runge-Kutta Method, commonly notated as (b)
    weights: Vec<f32>, 
    //<order>x<order> Matrix holding the coefficients for the Runge-Kutta Method, commonly notated as (a). 
    //matrix is lower triangular for explicit forms of the method
    coeff_matrix: Vec<Vec<f32>> 
}
impl DiffEqSolverConfig{
//Constructors:

    pub fn euler() -> Self {
        Self{
            order: 1,
            nodes: vec![0f32],
            weights: vec![1f32],
            coeff_matrix: vec![vec![0f32]]
        }
    }
    pub fn second_order(a: f32) -> Self{
        Self{
            order: 2,
            nodes: vec![0.0, a],
            weights: vec![1.0 - 1.0/(2.0*a), 1.0/(2.0*a)],
            coeff_matrix: vec![
                vec![0.0, 0.0],
                vec![a, 0.0]
            ]
        }
    }
    pub fn midpoint() -> Self {
        Self::second_order(0.5)
    }
    pub fn heun() -> Self{Self::second_order(1.0)}
    pub fn ralston() -> Self{Self::second_order(2.0/3.0)}
    pub fn third_order(a: f32) -> Self{
        assert_ne!(a, 1.0);
        assert_ne!(a, 0.0);
        assert_ne!(a, 2.0/3.0);
        Self{
            order: 3,
            nodes: vec![0.0, a, 1.0],
            weights: vec![
                0.5 - 1.0/(6.0*a),
                1.0/(6.0*a*(1.0-a)),
                (2.0-3.0*a)/(6.0 - 6.0*a)
            ],
            coeff_matrix: vec![
                vec![0.0, 0.0, 0.0],
                vec![a, 0.0, 0.0],
                vec![1.0 + (1.0-a)/(a*(3.0*a-2.0)), (a-1.0)/(a*(3.0*a-2.0)), 0.0]
            ]
        }
    }
    pub fn kutta_third() -> Self{
        Self { 
            order: 3, 
            nodes: vec![0.0, 0.5, 1.0], 
            weights: vec![1.0/6.0, 2.0/3.0, 1.0/6.0], 
            coeff_matrix: vec![
                vec![0.0, 0.0, 0.0],
                vec![0.5, 0.0, 0.0],
                vec![-1.0, 2.0, 0.0]
            ] 
        }
    }
    pub fn heun_third() -> Self{
        Self { 
            order: 3, 
            nodes: vec![0.0, 1.0/3.0, 2.0/3.0], 
            weights: vec![1.0/4.0, 0.0, 3.0/4.0], 
            coeff_matrix: vec![
                vec![0.0, 0.0, 0.0],
                vec![1.0/3.0, 0.0, 0.0],
                vec![0.0, 2.0/3.0, 0.0]
            ] 
        }
    }
    pub fn wray_third() -> Self{
        Self { 
            order: 3, 
            nodes: vec![0.0, 8.0/15.0, 2.0/3.0], 
            weights: vec![1.0/4.0, 0.0, 3.0/4.0], 
            coeff_matrix: vec![
                vec![0.0, 0.0, 0.0],
                vec![8.0/15.0, 0.0, 0.0],
                vec![1.0/4.0, 5.0/12.0, 0.0]
            ] 
        }
    }
    pub fn ralston_third() -> Self{
        Self { 
            order: 3, 
            nodes: vec![0.0, 0.5, 0.75], 
            weights: vec![2.0/9.0, 1.0/3.0, 4.0/9.0], 
            coeff_matrix: vec![
                vec![0.0, 0.0, 0.0],
                vec![1.0/2.0, 0.0, 0.0],
                vec![0.0, 3.0/4.0, 0.0]
            ] 
        }
    }
    pub fn ssprk3() -> Self{
        Self { 
            order: 3, 
            nodes: vec![0.0, 1.0, 0.5], 
            weights: vec![1.0/6.0, 1.0/6.0, 2.0/3.0], 
            coeff_matrix: vec![
                vec![0.0, 0.0, 0.0],
                vec![1.0, 0.0, 0.0],
                vec![1.0/4.0, 1.0/4.0, 0.0]
            ] 
        }
    }
    pub fn rk4() -> Self {
        Self{
            order: 4,
            nodes: vec![0.0, 0.5, 0.5, 1.0],
            weights: vec![1.0/6.0, 1.0/3.0, 1.0/3.0, 1.0/6.0],
            coeff_matrix: vec![
                vec![0.0, 0.0, 0.0, 0.0],
                vec![0.5, 0.0, 0.0, 0.0],
                vec![0.0, 0.5, 0.0, 0.0],
                vec![0.0, 0.0, 1.0, 0.0]
            ]
        }
    }
    pub fn rule3_8() -> Self {
        Self{
            order: 4,
            nodes: vec![0.0, 1.0/3.0, 2.0/3.0, 1.0],
            weights: vec![1.0/8.0, 3.0/8.0, 3.0/8.0, 1.0/8.0],
            coeff_matrix: vec![
                vec![0.0, 0.0, 0.0, 0.0],
                vec![1.0/3.0, 0.0, 0.0, 0.0],
                vec![-1.0/3.0, 1.0, 0.0, 0.0],
                vec![1.0, -1.0, 1.0, 0.0]
            ]
        }
    }
    pub fn ralston_fourth() -> Self {
        Self{
            order: 4,
            nodes: vec![0.0, 0.4, 0.45573725, 1.0],
            weights: vec![0.17476028, -0.55148066, 1.20553560, 0.17118478],
            coeff_matrix: vec![
                vec![0.0, 0.0, 0.0, 0.0],
                vec![0.4, 0.0, 0.0, 0.0],
                vec![0.29697761, 0.15875964, 0.0, 0.0],
                vec![0.21810040, -3.05096516, 3.83286476, 0.0]
            ]
        }
    }
}

pub fn UpdateRungeKutta(
    mut moving_items: Query<(&mut Transform, &mut SimulationData, &GlobalTransform, &RigidBody2D)>
){

}

#[derive(Component)]
pub struct SimulationData{}