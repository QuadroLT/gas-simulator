use bevy::prelude::*;
use egui_plot::Bar;

#[derive(Debug, Resource)]
pub struct SimulationData{
    pub reducer: f32,
    pub number_of_balls: i32,
    pub wall_temperature: f32,
    pub ball_temperature: f32,
    pub wall_interactions: bool,
}

impl Default for  SimulationData{
    fn default() -> Self{
        SimulationData{
            reducer: 0.5,
            number_of_balls: 5000,
            wall_temperature: 273.15,
            ball_temperature: 5.0,
            wall_interactions: true,
        }
    }
}


#[derive(Debug, Resource)]
pub struct BarPlotData{
    pub bars: Vec<Bar>,
}

impl Default for BarPlotData{
    fn default() -> Self {
        BarPlotData{
            bars: Vec::from([Bar::new(0.0, 0.0)]),
        }
    }
}
