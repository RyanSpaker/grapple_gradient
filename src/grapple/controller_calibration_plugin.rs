use super::*;

#[derive(Resource)]
pub struct RecordLeftStickToggle(bool);
impl Default for RecordLeftStickToggle{
    fn default() -> Self{
        RecordLeftStickToggle(false)
    }
}

#[derive(Resource)]
pub struct DualAxisInputStorage(Vec<(f32, f32)>);
impl Default for DualAxisInputStorage{
    fn default() -> Self {
        Self(vec![])
    }
}

#[derive(Resource)]
pub struct LeftAxisWarp{
    pub warp: f32,
}
impl Default for LeftAxisWarp{
    fn default() -> Self{
        LeftAxisWarp { warp: 1f32 }
    }
}

#[derive(Event)]
pub struct CalculateLeftAxisWarpEvent{}

pub struct ControllerCalibrationPlugin;
impl Plugin for ControllerCalibrationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DualAxisInputStorage>()
            .init_resource::<RecordLeftStickToggle>()
            .init_resource::<LeftAxisWarp>()
            .add_event::<CalculateLeftAxisWarpEvent>()
            .add_systems(Update, ( 
                update_rla_toggle, 
                record_left_axis.run_if(should_record_left_axis),
                calculate_left_axis_warp.run_if(on_event::<CalculateLeftAxisWarpEvent>())
            ));

    }
}

fn calculate_left_axis_warp(
    raw_input: Res<DualAxisInputStorage>,
    mut warp: ResMut<LeftAxisWarp>
){
    warp.warp = raw_input.0.iter().fold(10f32, |acc, val| {
        return if val.0*val.0+val.1*val.1<acc { val.0*val.0+val.1*val.1 } else { acc };
    }).sqrt().recip();
}

fn update_rla_toggle(
    input: Res<Input<KeyCode>>, 
    mut toggle: ResMut<RecordLeftStickToggle>, 
    mut storage: ResMut<DualAxisInputStorage>, 
    mut calc_warp: EventWriter<CalculateLeftAxisWarpEvent>,
){
    if input.just_pressed(KeyCode::Key0){
        toggle.0 = !toggle.0;
        if toggle.0{
            storage.0.clear();
        }else{
            calc_warp.send(CalculateLeftAxisWarpEvent{});
        }
    }
}

fn should_record_left_axis(toggle: Res<RecordLeftStickToggle>) -> bool{
    return toggle.0;
}

fn record_left_axis(
    gamepads: Res<Gamepads>, 
    input: Res<Axis<GamepadAxis>>, 
    mut storage: ResMut<DualAxisInputStorage>
){
    if let Some(gamepad) = gamepads.iter().next(){
        let axis_lx = GamepadAxis{gamepad: gamepad, axis_type: GamepadAxisType::LeftStickX};
        let axis_ly = GamepadAxis{gamepad: gamepad, axis_type: GamepadAxisType::LeftStickY};
        if let Some(y) = input.get(axis_ly) {
            if let Some(x) = input.get(axis_lx){
                storage.0.push((x, y));
            }
        }
    }
}