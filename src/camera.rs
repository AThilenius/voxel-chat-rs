use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use egui::style::Margin;

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
    pub margins: Margin,
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
        mouse_sensitive: 5.0,
        scroll_sensitive: 5.0,
        azimuth: 0.0,
        zenith: std::f32::consts::PI / 4.0,
        distance: 100.0,
        margins: default(),
    });
}

fn camera_pan_orbit(
    mut cameras: Query<(&mut Transform, &Projection), With<Camera>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_scroll: EventReader<MouseWheel>,
    mut camera_controller: ResMut<CameraController>,
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    global_transforms: Query<&GlobalTransform>,
) {
    let mouse_mul = camera_controller.mouse_sensitive * 0.001;
    let scroll_mul = camera_controller.scroll_sensitive * 0.0002;

    let scroll = mouse_scroll.iter().fold(0.0, |v, e| v + e.y);
    camera_controller.distance = (camera_controller.distance
        - (scroll * camera_controller.distance * scroll_mul))
        .clamp(0.1, 2000.0);

    if mouse_button_input.pressed(MouseButton::Right) {
        let pan = mouse_motion.iter().fold(Vec2::ZERO, |v, e| v + e.delta) * mouse_mul;

        camera_controller.azimuth += -pan.x;
        camera_controller.zenith = (camera_controller.zenith + pan.y).clamp(
            -(std::f32::consts::PI / 2.0) + 0.001,
            (std::f32::consts::PI / 2.0) - 0.001,
        );
    }

    for (mut camera_transform, camera_projection) in cameras.iter_mut() {
        let global_target = match camera_controller.target {
            CameraTarget::Entity(e) => global_transforms
                .get(e)
                .ok()
                .unwrap_or(&GlobalTransform::IDENTITY)
                .translation(),
            CameraTarget::Point(p) => p,
        };

        // The dolly transform
        let mut transform = Transform::from_translation(
            convert_polar_to_cartesian(
                camera_controller.azimuth,
                camera_controller.zenith,
                camera_controller.distance,
            ) + global_target,
        );

        transform.look_at(global_target, Vec3::Y);

        // Then modify that for margins
        match camera_projection {
            Projection::Perspective(projection) => {
                let distance_to_target = (global_target - transform.translation).length();
                let frustum_height = 2.0 * distance_to_target * (projection.fov * 0.5).tan();
                let frustum_width = frustum_height * projection.aspect_ratio;

                let window = windows.get_primary().unwrap();

                let left_taken = camera_controller.margins.left / window.width();
                let right_taken = camera_controller.margins.right / window.width();
                let top_taken = camera_controller.margins.top / window.height();
                let bottom_taken = camera_controller.margins.bottom / window.height();
                transform.translation = transform.translation
                    + transform.rotation.mul_vec3(Vec3::new(
                        (right_taken - left_taken) * frustum_width * 0.5,
                        (top_taken - bottom_taken) * frustum_height * 0.5,
                        0.0,
                    ));
            }
            _ => {}
        }

        *camera_transform = transform;
    }
}

fn convert_polar_to_cartesian(azimuth: f32, zenith: f32, distance: f32) -> Vec3 {
    let x = azimuth.sin() * zenith.cos();
    let y = zenith.sin();
    let z = azimuth.cos() * zenith.cos();

    Vec3::new(x, y, z) * distance
}
