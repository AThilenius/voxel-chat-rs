use std::f32::consts::*;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use bevy_rapier3d::prelude::*;
use camera::CameraPlugin;
use editor::EditorPlugin;
use resize::ResizePlugin;
use voxel::VoxelMaterial;

#[macro_use]
mod macros;
mod camera;
mod editor;
mod net;
mod resize;
mod serde_test;
mod voxel;

pub fn core_main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(EguiPlugin)
        .add_plugin(ResizePlugin)
        .add_plugin(EditorPlugin)
        .add_plugin(CameraPlugin)
        .add_startup_system(setup_world_and_camera)
        .add_plugin(MaterialPlugin::<VoxelMaterial>::default())
        .add_system(draw_world_debug_lines)
        .run();
}

fn setup_world_and_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut stdmats: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, PI / 4.0))
            * Transform::from_rotation(Quat::from_axis_angle(Vec3::X, -PI / 4.0)),
        ..default()
    });
}

fn draw_world_debug_lines(mut lines: ResMut<DebugLines>) {
    lines.line_colored(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), f32::MAX, Color::RED);
    lines.line_colored(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0), f32::MAX, Color::BLUE);
    lines.line_colored(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), f32::MAX, Color::GREEN);
}

fn setup_vox_mesh(
    mut commands: Commands,
    mut stdmats: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    commands.spawn(PbrBundle {
        transform: Transform::from_scale((0.02, 0.02, 0.02).into())
            * Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, PI)),
        mesh: assets.load("chicken.vox"),
        material: stdmats.add(Color::rgb(1., 1., 1.).into()),
        ..Default::default()
    });
}

fn setup_physics(mut commands: Commands) {
    /* Create the ground. */
    commands.spawn((
        Collider::cuboid(100.0, 1.0, 100.0),
        TransformBundle::from(Transform::from_xyz(0.0, -1.0, 0.0)),
    ));

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5, 0.5),
        TransformBundle::from(Transform::from_xyz(0.0, 1.0, 0.0)),
        ExternalImpulse::default(),
    ));

    /* Create the bouncing ball. */
    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Restitution::coefficient(0.7),
        Velocity {
            angvel: Vec3::new(0.2, 0.4, 10.0),
            ..default()
        },
        Damping {
            angular_damping: 2.0,
            ..default()
        },
        TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)),
        ExternalImpulse::default(),
    ));
}

fn apply_force_at_raycast(
    mouse_button_input: Res<Input<MouseButton>>,
    rapier_context: Res<RapierContext>,
    windows: Res<Windows>,
    mut lines: ResMut<DebugLines>,
    camera: Query<(&GlobalTransform, &Camera)>,
    global_transforms: Query<&GlobalTransform>,
    mut impulses: Query<&mut ExternalImpulse>,
) {
    let (transform, camera) = camera.single();
    let window = unwrap_or_return!(windows.get_primary());
    let cursor = unwrap_or_return!(window.cursor_position());

    let left = mouse_button_input.just_pressed(MouseButton::Left);
    let right = mouse_button_input.just_pressed(MouseButton::Right);

    if left || right {
        let ray = camera.viewport_to_world(transform, cursor).unwrap();

        if let Some((entity, intersection)) = rapier_context.cast_ray_and_get_normal(
            ray.origin,
            ray.direction,
            100.0,
            true,
            QueryFilter::only_dynamic(),
        ) {
            let position = ok_or_return!(global_transforms.get(entity))
                .affine()
                .translation;

            let normal = if left {
                -intersection.normal
            } else {
                intersection.normal
            };

            if let Ok(mut impulse) = impulses.get_mut(entity) {
                *impulse = ExternalImpulse::at_point(normal, intersection.point, position.into());
            }

            lines.line(intersection.point, intersection.point + normal, 4.0);
        }
    }
}

fn setup_animation(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut animations: ResMut<Assets<AnimationClip>>,
) {
    let named = Name::new("named");
    let mut animation = AnimationClip::default();

    // A curve can modify a single part of a transform, here the translation
    animation.add_curve_to_path(
        EntityPath {
            parts: vec![named.clone()],
        },
        VariableCurve {
            keyframe_timestamps: vec![0.0, 2.0, 2.5, 3.0, 5.0],
            keyframes: Keyframes::Rotation(vec![
                Quat::IDENTITY,
                Quat::from_axis_angle(Vec3::Y, PI / 2.),
                Quat::from_axis_angle(Vec3::Y, PI / 2. * 2.),
                Quat::from_axis_angle(Vec3::Y, PI / 2. * 3.),
                Quat::IDENTITY,
            ]),
        },
    );

    let mut player = AnimationPlayer::default();
    player.play(animations.add(animation)).repeat();

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::default())),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            ..default()
        },
        named,
        player,
    ));
}
