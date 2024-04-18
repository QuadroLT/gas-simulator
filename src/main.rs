
use bevy::prelude::*;


use bevy::sprite::{Mesh2dHandle, MaterialMesh2dBundle};
use bevy::math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume};


use bevy_egui::{egui, EguiContexts, EguiPlugin};
use egui_plot::{Bar, BarChart, Plot};


// use bevy_rapier2d::parry::query;
use rand::prelude::*;
use rand_distr::num_traits::{Pow, ToPrimitive};
use rand_distr::{Normal, Uniform, ChiSquared};

const NUMBER_OF_DOTS: i32 = 5000; // TODO remove hardcoding
const BALL_RADIUS: f32 = 3.0; // TODO remove hardcoding
const AVOGADRO: f32 = 6.02214e23;
const BOLZMAN: f32 = 1.38065e-23;
const WALL_LEFT: f32 = -570.0;
const WALL_RIGHT: f32 = 570.0;
const WALL_TOP: f32 = 350.0;
const WALL_BOTTOM: f32 = -350.0;
const WALL_THIKNESS: f32 = 30.0;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const REDUCER: f32 = 0.2;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(PostStartup, create_graph_data)
        .add_systems(Update, (
            check_for_wall_collision,
            check_between_ball_collisions,
            update_positions,
            update_graph_data,
            ui_example_system,
        ))
        .run();
}


// #[derive(Component)]
enum WallLocation{
    Left,
    Right,
    Top,
    Bottom,
}

impl WallLocation {
    fn position(&self) -> Position {
        match self {
            WallLocation::Left => Position::new(WALL_LEFT, 0.0),
            WallLocation::Right => Position::new(WALL_RIGHT, 0.0),
            WallLocation::Top => Position::new(0.0, WALL_TOP),
            WallLocation::Bottom => Position::new(0.0, WALL_BOTTOM),
        }
    }

    fn size(&self) -> Vec3 {
        let arena_hight = WALL_TOP - WALL_BOTTOM;
        let arena_width = WALL_RIGHT - WALL_LEFT;
        match self {
            WallLocation::Top | WallLocation::Bottom => Vec3::new(arena_width + WALL_THIKNESS,WALL_THIKNESS, 1.0),
            WallLocation::Left | WallLocation::Right => Vec3::new( WALL_THIKNESS, arena_hight + WALL_THIKNESS, 1.0),
        }
    }
}

#[derive(Component)]
struct WallTemperature{
    value: f32,
}

impl WallTemperature{
    fn new(value: f32) ->Self{
        WallTemperature{value}
    }
}

#[derive(Component, Debug)]
struct BallTemperature{
    value: f32,
}

impl BallTemperature{
    fn new(value: f32) -> Self{
        BallTemperature{value}
    }
}

#[derive(Bundle)]
struct WallBundle{
    sprite_bundle: SpriteBundle,
    wall: Wall,
    temperature: WallTemperature,
}

impl WallBundle{
    fn new(location: WallLocation, temperature: WallTemperature ) -> Self {
        WallBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                color: WALL_COLOR,
                ..default()},
                transform: Transform {
                    translation: location.position().value,
                    scale: location.size(),
                    ..default()},
                ..default()
            },
            wall: Wall,
            temperature,
        }
    }
}


#[derive(Component, Debug)]
struct Ball;

#[derive(Debug, Component, Eq, PartialEq)]
enum Molecule {
    Methane,
    Oxygen,
    Formaldehyde,
    CarbonDioxide,
}

impl Ball{
    fn get_bounding_circle(ball_transform: &Transform) -> BoundingCircle{
        BoundingCircle::new(
            ball_transform.translation.truncate(),
            BALL_RADIUS,
        )
    }
}

#[derive(Component, Debug)]
struct Wall;
impl Wall {
    fn get_bounding_box(wall_transform: &Transform) -> Aabb2d{
        Aabb2d::new(
            wall_transform.translation.truncate(),
            wall_transform.scale.truncate(),
        )
    }
}


#[derive(Component, Debug)]
struct Position{
    value: Vec3,
}

impl Position{
    fn new(x:  f32, y: f32) -> Self{
        Position{
            value: Vec3::new(x, y, 0.0)
        }
    }

    fn random() -> Self{
        let dist_x = Normal::new(0.0, 150.0).unwrap();
        let dist_y = Normal::new(0.0, 70.0).unwrap();

        Position {
            value: Vec3 {
                x: (thread_rng().sample(dist_x)),
                y: (thread_rng().sample(dist_y)),
                z: 0.0
            }
        }
    }
}


#[derive(Component, Debug)]
struct Velocity{
    value: Vec3,
}

impl Velocity{
    fn random(mass: f32, temperature: f32)-> Self{
        let uniform: Uniform<f32> = Uniform::new(0.0, 1.0);
        let center: f32 = abs_velocity_from_energy(mass, temperature);
        let normal = ChiSquared::new(center).unwrap();
        let theta = 2.0 * std::f32::consts::PI * thread_rng().sample(uniform);
        let length = thread_rng().sample(normal)* REDUCER;
        let x = theta.cos() * length; 
        let y = theta.sin() * length;
        Velocity {
            value: Vec3 { x: (x), y: (y), z: (0.0) }
        }
    }
}

fn abs_velocity_from_energy(mass: f32, temperature: f32) -> f32{
    ((4.0 * BOLZMAN * temperature) /( mass * 3.0)).sqrt()
}

#[derive(Component, Debug)]
struct Mass{
    value: f32,
}

#[derive(Bundle, Debug)]
struct BallBundle{
    // position: Position,
    velocity: Velocity,
    mass: Mass,
    molecule: Molecule,
    temperature: BallTemperature,
}

impl BallBundle{
    fn new(molecule: Molecule, temperature: BallTemperature) -> Self {
        let mass: f32;
        match molecule {
            Molecule::Oxygen => mass = 32.0 / AVOGADRO / 1000.0,
            Molecule::Methane => mass = 16.0 / AVOGADRO / 1000.0,
            Molecule::CarbonDioxide => mass = 44.0 / AVOGADRO / 1000.0,
            Molecule::Formaldehyde => mass = 30.0 / AVOGADRO / 1000.0
        }
        BallBundle {
            velocity: Velocity::random(mass, temperature.value),
            mass: Mass {
                value: mass,
            },
            molecule,
            temperature,
        }
    }
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn(Camera2dBundle::default());
    let mut shapes = Vec::new();
    for _i in 0..NUMBER_OF_DOTS {
        shapes.push(Mesh2dHandle(meshes.add(Circle {radius: BALL_RADIUS})))
   }

    for (_i, shape) in shapes.into_iter().enumerate(){
        let mut color = Color::rgb(1.0, 0.0, 0.0);
        let mut molecule = Molecule::Methane;
        let temperature = BallTemperature::new(10.0);
        if _i % 2 == 0 {
            color = Color::rgb(1.0, 1.0, 0.0);
            molecule = Molecule::Oxygen;
        }
        commands.spawn((
            BallBundle::new(molecule, temperature),
            MaterialMesh2dBundle {
                mesh: shape,
                material: materials.add(color),
                transform: Transform::from_translation(Position::random().value),
                ..default()
            },
            Ball,
        ));
    };
    commands.spawn(WallBundle::new(WallLocation::Bottom, WallTemperature::new(2730.15))); // TODO remove hardcoding
    commands.spawn(WallBundle::new(WallLocation::Top, WallTemperature::new(2720.15)));
    commands.spawn(WallBundle::new(WallLocation::Left, WallTemperature::new(2730.15)));
    commands.spawn(WallBundle::new(WallLocation::Right, WallTemperature::new(2730.15)));
}

fn update_positions( mut items: Query<(&mut Transform, &Velocity), With<Ball>>,
                     time: Res<Time>
){
    let dt = time.delta_seconds();
    for (mut transform, velocity) in &mut items {
        transform.translation.x  += velocity.value.x * dt;
        transform.translation.y  += velocity.value.y * dt;
        transform.translation.z  += velocity.value.z * dt;
    }
}



// #[derive(Debug, Event)]
// struct CollisionEvent;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Collision{
    Left,
    Right,
    Top,
    Bottom,
}


fn collide_with_wall(ball: BoundingCircle, wall: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&wall) {
        return None;
    }

    let closest = wall.closest_point(ball.center());
    let offset = ball.center - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };
    Some(side)
}


fn get_velocity_from_temperature(initial: &Vec3, temperature: &f32, mass: &f32) -> Vec3{
    let initial_length = initial.length();
    let center = abs_velocity_from_energy(*mass, *temperature);
    let normal = ChiSquared::new(center).expect(format!("Got parameter {}", center).as_str());
    let end_length = thread_rng().sample(normal) * REDUCER;
    let ratio = end_length / initial_length;
    Vec3::new(initial.x * ratio, initial.y * ratio, initial.z * ratio)
}


fn check_for_wall_collision(
     // mut commands: Commands,
     mut ball_query: Query<(&mut Velocity, &Transform, &mut BallTemperature, &Mass), With<Ball>>,
     wall_query: Query<(&WallTemperature, &Transform), With<Wall>>,
 ){
     for (mut ball_velocity,
          ball_transform,
          mut ball_temp,
          mass) in ball_query.iter_mut(){
         let ball_boundary = Ball::get_bounding_circle(ball_transform);
         for (wall_temp, wall_transform) in  wall_query.iter(){
             let wall_boundary = Wall::get_bounding_box(wall_transform);
             let collision_opt = collide_with_wall(ball_boundary, wall_boundary);
             if let Some(collision) = collision_opt {
                 // collision_events.send_default();
                 let mut reflect_x = false;
                 let mut reflect_y = false;
                 match collision {
                     Collision::Left => reflect_x = ball_velocity.value.x > 0.0,
                     Collision::Right => reflect_x = ball_velocity.value.x < 0.0,
                     Collision::Top => reflect_y = ball_velocity.value.y < 0.0,
                     Collision::Bottom => reflect_y = ball_velocity.value.y > 0.0,
                 }
                 if reflect_x{
                     ball_velocity.value.x = -ball_velocity.value.x;
                 }
                 if reflect_y{
                     ball_velocity.value.y = -ball_velocity.value.y;
                 }
                 ball_temp.value = (ball_temp.value + wall_temp.value) / 2.0;
                 let new_velocity = get_velocity_from_temperature(&ball_velocity.value, &ball_temp.value, &mass.value);
                 ball_velocity.value = new_velocity;
             }
         }
     }
 }

fn broad_phase_collision(
    query: &Query<(&mut Velocity, &Transform, &Mass, Entity, &mut BallTemperature),
                  With<Ball>>
) -> Vec<(Entity, Entity)>{
    let mut ball_vec = query
        .into_iter()
        .collect::<Vec<_>>();
    // sweep on x
    ball_vec.sort_by(
        |ball1, ball2| {
            ball1.1.translation.x.partial_cmp(&ball2.1.translation.x).expect(
                format!("Got ball1 {} ball2 {}", ball1.1.translation.x, ball2.1.translation.x).as_str())
        });
    let mut balls_to_update = Vec::new();
    for i in 0..ball_vec.len() - 1 {
        let ball1 = ball_vec[i].1.translation;
        let ball2 = ball_vec[i+1].1.translation;
        let item1  = ball1.x + BALL_RADIUS;
        let item2 = ball2.x - BALL_RADIUS;
        if item2 <= item1 {
            // println!(" candicadtes: {} {}", i, i+1);
            let x_proj = (ball1.x - ball2.x).abs();
            let y_proj = (ball1.y - ball2.y).abs();
            if x_proj.pow(2.0) + y_proj.pow(2.0) <= 2.0 * BALL_RADIUS.pow(2.0){
                balls_to_update.push((ball_vec[i].3, ball_vec[i+1].3));
            }
        }
    }
    balls_to_update

}

fn get_after_colition_velocities(
    ball1_transform: &Transform,
    ball1_velocity: &Mut<'_, Velocity>,
    ball1_mass: &Mass,
    ball2_transform: &Transform,
    ball2_velocity: &Mut<'_,Velocity>,
    ball2_mass: &Mass,
)  -> (Vec3, Vec3){
    // finding new colision space
    // unit normal vector
    let un = (ball2_transform.translation - ball1_transform.translation) / (ball2_transform.translation - ball1_transform.translation).length();
    // unit tangent vector
    let ut = Vec3::new(un.y * -1.0, un.x , un.z);
    // concersion of initial velocities into collision space
    let v1n = un.dot(ball1_velocity.value);
    let v2n = un.dot(ball2_velocity.value);
    let v1t = ut.dot(ball1_velocity.value);
    let v2t = ut.dot(ball2_velocity.value);
    // calculating after collision velocities in collision space
    // geting unit normal lengths 
    let v1n_a = (v1n * (ball1_mass.value - ball2_mass.value) + 2.0 * ball2_mass.value * v2n) / (ball1_mass.value + ball2_mass.value);
    let v2n_a =(v2n * (ball2_mass.value - ball1_mass.value) + 2.0 * ball1_mass.value * v1n) / (ball1_mass.value + ball2_mass.value);
    // getting unit normal vectors
    let v1n_a = v1n_a * un;
    let v2n_a = v2n_a * un;
    // getting unit tangent vectors
    let v1t_a = v1t * ut;
    let v2t_a = v2t * ut;
    // returning to standard space
    let v1_a = v1n_a + v1t_a;
    let v2_a = v2n_a + v2t_a;
    (v1_a, v2_a)
}


fn check_between_ball_collisions(
    mut ball_query: Query<(&mut Velocity, &Transform, &Mass, Entity, &mut BallTemperature), With<Ball>>)
{
    let targets = broad_phase_collision(&ball_query);
    for (b1, b2) in targets.into_iter(){
        let [mut ball1, mut ball2] = ball_query.get_many_mut([b1, b2]).unwrap();
        // let ball2 = ball_query.get(b2).unwrap();
        let (v1, v2) = get_after_colition_velocities(&ball1.1, &ball1.0, &ball1.2, ball2.1, &ball2.0, &ball2.2);
        ball1.0.value = v1;
        ball2.0.value = v2;
        ball1.4.value = (3.0 * ball1.2.value * v1.length().pow(2.0))/ (4.0 * BOLZMAN);
        ball2.4.value = (3.0 * ball2.2.value * v1.length().pow(2.0))/ (4.0 * BOLZMAN);
    }
}


#[derive(Debug, Component)]
struct BarPlotComponent{
    bars: Vec<Bar>,
}

impl BarPlotComponent{
    fn new () -> Self{
        BarPlotComponent{
            bars: Vec::from([Bar::new(0.0, 0.0)]),
        }
    }
}

#[derive(Debug, Bundle)]
struct UIDataBundle{
    hist_data: BarPlotComponent,
}



impl UIDataBundle {
    fn new() -> Self{
        UIDataBundle{
            hist_data: BarPlotComponent::new(),
        }
    } 
}



fn create_graph_data(
    mut commands: Commands,
){
    commands.spawn(UIDataBundle::new());
}

fn update_graph_data(
    mut graph_data: Query<&mut BarPlotComponent>,
    ball_data: Query<&BallTemperature>,
){
    let number_of_bins = 100;
    let ball_temperature = ball_data
        .iter()
        .map(|item| item.value.to_f64().unwrap())
        .collect::<Vec<_>>();

    let mut min = ball_temperature
        .clone()
        .into_iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max = ball_temperature
        .clone()
        .into_iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let step = (max - min) /number_of_bins.to_f64().unwrap();
    let mut bars = Vec::new();
    println!("--------------------------------frame--------------------------------");
    for _i in 0..number_of_bins{
        let count = ball_temperature.clone().into_iter().filter(|x| (x > &min) & (x <= &(min + step))).collect::<Vec<_>>().len().to_f64().unwrap();
        let val = (min + step) / 2.0;
        println!("x={val} y={count}");
        bars.push(Bar::new(val, count));
        // intervals.push((min, min + step));
        min = min + step;
        // println!("{min}");
    }
    let mut data = graph_data.get_single_mut().unwrap();
    data.bars = bars;
}


fn ui_example_system(
    mut contexts: EguiContexts,
    histogram: Query<&BarPlotComponent>,
){
    
    egui::Window::new("Controls").show(contexts.ctx_mut(),|ui: &mut egui::Ui| {
        ui.label("Particle temperature distribution");
        Plot::new("my_plot").view_aspect(2.0).show(ui, |plotui| plotui.bar_chart(
            BarChart::new(histogram.get_single().unwrap().bars.clone())
        ));
        ui.button("test").clicked();
    });
}





