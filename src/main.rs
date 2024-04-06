
// use std::collections::VecDeque;

use bevy::prelude::*;
// use bevy_rapier2d::prelude::*;

use bevy::sprite::{Mesh2dHandle, MaterialMesh2dBundle};
use bevy::math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume};


// use bevy_egui::{egui, EguiContexts, EguiPlugin};


// use bevy_rapier2d::parry::query;
use rand::prelude::*;
use rand_distr::num_traits::Pow;
use rand_distr::{Normal, Uniform};

const NUMBER_OF_DOTS: i32 = 400; // TODO remove hardcoding
const BALL_RADIUS: f32 = 5.0; // TODO remove hardcoding
const BALL_MASS: f32 = 2.0; // TODO remove hardcoding
const WALL_LEFT: f32 = -570.0;
const WALL_RIGHT: f32 = 570.0;
const WALL_TOP: f32 = 350.0;
const WALL_BOTTOM: f32 = -350.0;
const WALL_THIKNESS: f32 = 10.0;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // EguiPlugin,
        ))
        // .insert_resource(RapierConfiguration {
        //     gravity: Vec2::ZERO,
        //     ..Default::default()
        // })
        // .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1000000.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update_positions,
            check_for_wall_collision,
            check_between_ball_collisions,
            // ui_example_system,
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

#[derive(Bundle)]
struct WallBundle{
    sprite_bundle: SpriteBundle,
    wall: Wall,
}

impl WallBundle{
    fn new(location: WallLocation) -> Self {
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
            wall: Wall
        }
    }
}


#[derive(Component, Debug)]
struct Ball;

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
    fn random()-> Self{
        let uniform: Uniform<f32> = Uniform::new(0.0, 1.0);
        let normal: Normal<f32> = Normal::new(100.0, 1.0).unwrap(); // TODO remove hardcoding
        let theta = 2.0 * std::f32::consts::PI * thread_rng().sample(uniform);
        let x = theta.sin() * thread_rng().sample(normal);
        let y = theta.cos() * thread_rng().sample(normal);
        Velocity {
            value: Vec3 { x: (x), y: (y), z: (0.0) }
        }
    }
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
}

impl BallBundle{
    fn new() -> Self {
        BallBundle { velocity: Velocity::random(), mass: Mass { value: BALL_MASS } }
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
        let color = Color::Rgba { red: (1.), green: (1.), blue: (0.), alpha: (1.0) };
        commands.spawn((
            BallBundle::new(),
            MaterialMesh2dBundle {
                mesh: shape,
                material: materials.add(color),
                transform: Transform::from_translation(Position::random().value),
                ..default()
            },
            Ball,
        ));
        // .insert(RigidBody::Dynamic)
        // .insert(Collider::ball(BALL_RADIUS))
        // .insert(Ccd::enabled());
    };
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
}

fn update_positions( mut items: Query<(&mut Transform, &Velocity)>, time: Res<Time>){
    let dt = time.delta_seconds();
    for (mut transform, velocity) in &mut items {
        transform.translation.x  += velocity.value.x * dt;
        transform.translation.y  += velocity.value.y * dt;
        transform.translation.z  += velocity.value.z * dt;

    }
}



#[derive(Debug, Event)]
struct CollisionEvent;

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


 fn check_for_wall_collision(
     // mut commands: Commands,
     mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
     wall_query: Query<(Entity, &Transform), With<Wall>>,
     // collision_events: EventWriter<CollisionEvent>,
 ){
     for (mut ball_velocity, ball_transform) in ball_query.iter_mut(){
         let ball_boundary = Ball::get_bounding_circle(ball_transform);
         for (_wall, wall_transform) in  wall_query.iter(){
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
             }
         }
     }
 }

fn broad_phase_collision(query: &Query<(&mut Velocity, &Transform, &Mass, Entity), With<Ball>>) -> Vec<(Entity, Entity)>{
    let mut ball_vec = query
        .into_iter()
        .collect::<Vec<_>>();
    // sweep on x
    ball_vec.sort_by(
        |ball1, ball2| ball1.1.translation.x.partial_cmp(&ball2.1.translation.x).unwrap()
    );
    let mut balls_to_update = Vec::new();
    for i in 0..ball_vec.len() - 1 {
        let ball1 = ball_vec[i].1.translation;
        // let ball1v = ball_vec[i].0.value;
        // let ball2v = ball_vec[i+1].0.value;
        // let ball1m = ball_vec[i].2.value;
        // let ball2m = ball_vec[i+1].2.value;
        let ball2 = ball_vec[i+1].1.translation;
        let item1  = ball1.x + BALL_RADIUS;
        let item2 = ball2.x - BALL_RADIUS;
        if item2 < item1 {
            // println!(" candicadtes: {} {}", i, i+1);
            let x_proj = (ball1.x - ball2.x).abs();
            let y_proj = (ball1.y - ball2.y).abs();
            if x_proj.pow(2.0) + y_proj.pow(2.0) <= 2.0*BALL_RADIUS.pow(2.0){
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
    mut ball_query: Query<(&mut Velocity, &Transform, &Mass, Entity), With<Ball>> ){
    let targets = broad_phase_collision(&ball_query);
    for (b1, b2) in targets.into_iter(){
        let mut balls = ball_query.get_many_mut([b1, b2]).unwrap();
        // let ball2 = ball_query.get(b2).unwrap();
        let (v1, v2) = get_after_colition_velocities(&balls[0].1, &balls[0].0, &balls[0].2, balls[1].1, &balls[1].0, &balls[1].2);
        balls[0].0.value = v1;
        balls[1].0.value = v2;

    }
}





// fn ui_example_system(mut contexts: EguiContexts) {
//     egui::Window::new("Controls").show(contexts.ctx_mut(), |ui| {
//         ui.label("Particles");
//         ui.label("Energies");
//         ui.button("test").clicked();
//     });
// }





