use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_camera)
            .add_system(camera_pan_orbit);
    }
}

#[derive(Resource)]
pub struct CameraController {
    pub target: CameraTarget,
    pub mouse_sensitive: f32,
    pub scroll_sensitive: f32,
    pub azimuth: f32,
    pub zenith: f32,
    pub distance: f32,
}

pub enum CameraTarget {
    Entity(Entity),
    Point(Vec3),
}

fn setup_camera(mut commands: Commands) {
    // TODO: Switch on HDR when it's stable.
    commands.spawn((Camera3dBundle {
        camera: Camera { ..default() },
        transform: Transform::from_xyz(10.0, 5.0, 5.0)
            .looking_at(Vec3::new(2.0, 2.0, 0.0), Vec3::Y),
        ..default()
    },));

    commands.insert_resource(CameraController {
        target: CameraTarget::Point(Vec3::ZERO),
        mouse_sensitive: 10.0,
        scroll_sensitive: 10.0,
        azimuth: 0.0,
        zenith: std::f32::consts::PI / 4.0,
        distance: 10.0,
    });
}

fn camera_pan_orbit(
    mut cameras: Query<&mut Transform, With<Camera>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_scroll: EventReader<MouseWheel>,
    mut camera_dolly: ResMut<CameraController>,
    mouse_button_input: Res<Input<MouseButton>>,
    global_transforms: Query<&GlobalTransform>,
) {
    let mouse_mul = camera_dolly.mouse_sensitive * 0.001;
    let scroll_mul = camera_dolly.scroll_sensitive * 0.0002;

    let scroll = mouse_scroll.iter().fold(0.0, |v, e| v + e.y);
    camera_dolly.distance =
        (camera_dolly.distance - (scroll * camera_dolly.distance * scroll_mul)).clamp(0.1, 200.0);

    if mouse_button_input.pressed(MouseButton::Right) {
        let pan = mouse_motion.iter().fold(Vec2::ZERO, |v, e| v + e.delta) * mouse_mul;

        camera_dolly.azimuth += -pan.x;
        camera_dolly.zenith = (camera_dolly.zenith + pan.y).clamp(
            -(std::f32::consts::PI / 2.0) + 0.001,
            (std::f32::consts::PI / 2.0) - 0.001,
        );
    }

    for mut camera_transform in cameras.iter_mut() {
        let global_target = match camera_dolly.target {
            CameraTarget::Entity(e) => global_transforms
                .get(e)
                .ok()
                .unwrap_or(&GlobalTransform::IDENTITY)
                .translation(),
            CameraTarget::Point(p) => p,
        };

        *camera_transform = Transform::from_translation(
            convert_polar_to_cartesian(
                camera_dolly.azimuth,
                camera_dolly.zenith,
                camera_dolly.distance,
            ) + global_target,
        );

        camera_transform.look_at(global_target, Vec3::Y);
    }
}

fn convert_polar_to_cartesian(azimuth: f32, zenith: f32, distance: f32) -> Vec3 {
    let x = azimuth.sin() * zenith.cos();
    let y = zenith.sin();
    let z = azimuth.cos() * zenith.cos();

    Vec3::new(x, y, z) * distance
}
