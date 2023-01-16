use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{
    color::Hsva,
    color_picker::{color_picker_hsva_2d, Alpha},
    Slider,
};

use crate::{camera::CameraController, voxel::Rgba};

use super::EditorResource;

pub fn editor_ui(
    mut voxel_editor: ResMut<EditorResource>,
    mut egui_context: ResMut<EguiContext>,
    mut camera_controller: ResMut<CameraController>,
) {
    camera_controller.margins.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            let color = Color::from(voxel_editor.material.color).as_rgba_f32();
            let mut hsva = Hsva::from_rgb([color[0], color[1], color[2]]);
            color_picker_hsva_2d(ui, &mut hsva, Alpha::Opaque);
            let color = hsva.to_rgb();
            voxel_editor.material.color = Rgba::from(Color::rgb(color[0], color[1], color[2]));

            ui.add(
                Slider::new(&mut voxel_editor.material.metallic, (0)..=(255))
                    .smart_aim(true)
                    .text("Metallic"),
            );

            ui.add(
                Slider::new(&mut voxel_editor.material.roughness, (0)..=(255))
                    .smart_aim(true)
                    .text("Roughness"),
            );

            ui.add(
                Slider::new(&mut voxel_editor.material.emission, (0)..=(255))
                    .smart_aim(true)
                    .text("Emission"),
            );

            ui.add(
                Slider::new(&mut voxel_editor.material.reflectance, (0)..=(255))
                    .smart_aim(true)
                    .text("Reflectance"),
            );

            ui.label(format!("Entity: {:?}", voxel_editor.entity));
            ui.label(format!("Prefab Entity: {:?}", voxel_editor.prefab_entity));
            ui.label(format!("Buffer dirty: {}", voxel_editor.buffer_dirty));
            ui.label(format!("Undo stack: {}", voxel_editor.undo_stack.len()));
        })
        .response
        .rect
        .width();
}
