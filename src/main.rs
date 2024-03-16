
use bevy::ecs::reflect;
use bevy::prelude::*;
use bevy::sprite::{Mesh2dHandle, MaterialMesh2dBundle};
use bevy::math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume};

use rand::prelude::*;
use rand_distr::{Normal, Uniform};

const NUMBER_OF_DOTS: i32 = 200;
const BALL_RADIUS: f32 = 5.0;
const WALL_LEFT: f32 = -570.0;
const WALL_RIGHT: f32 = 570.0;
const WALL_TOP: f32 = 350.0;
const WALL_BOTTOM: f32 = -350.0;
const WALL_THIKNESS: f32 = 10.0;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (update_positions, check_for_wall_collision))
        .run();
}

#[derive(Component)]
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
    fn new(location: WallLocation) -> Self{
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


#[derive(Component)]
struct Ball;

impl Ball{
    fn get_bounding_circle(ball_transform: &Transform) -> BoundingCircle{
        BoundingCircle::new(
            ball_transform.translation.truncate(),
            BALL_RADIUS,
        )
    }
}

#[derive(Component)]
struct Wall;
impl Wall {
    fn get_bounding_box(wall_transform: &Transform) -> Aabb2d{
        Aabb2d::new(
            wall_transform.translation.truncate(),
            wall_transform.scale.truncate(),
        )
    }
}


#[derive(Component)]
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

#[derive(Component)]
struct Velocity{
    value: Vec3,
}

impl Velocity{
    fn random()-> Self{
        let uniform: Uniform<f32> = Uniform::new(0.0, 1.0);
        let normal: Normal<f32> = Normal::new(100.0, 1.0).unwrap();
        let theta = 2.0 * std::f32::consts::PI * thread_rng().sample(uniform);
        let x = theta.sin() * thread_rng().sample(normal);
        let y = theta.cos() * thread_rng().sample(normal);
        Velocity {
            value: Vec3 { x: (x), y: (y), z: (0.0) }
        }
    }
}

#[derive(Bundle)]
struct BallBundle{
    // position: Position,
    velocity: Velocity,
}

impl BallBundle{
    fn new() -> Self {
        BallBundle { velocity: Velocity::random() }
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

#[derive(PartialEq, Eq, Clone, Copy)]
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

    let closest = wall.closest_point(ball.center);
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
         for (wall, wall_transform) in  wall_query.iter(){
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

