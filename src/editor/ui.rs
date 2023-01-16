use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{
    color::Hsva,
    color_picker::{color_picker_hsva_2d, Alpha},
    CollapsingHeader, Slider, Ui,
};

use crate::{camera::CameraController, voxel::Rgba};

use super::{entity_buffer::EntityBuffer, EditorResource};

pub fn editor_ui(
    mut voxel_editor: ResMut<EditorResource>,
    mut egui_context: ResMut<EguiContext>,
    mut camera_controller: ResMut<CameraController>,
    ui_query: Query<(&Name, Option<&Children>, Option<&EntityBuffer>)>,
    entity_buffers: Query<&EntityBuffer>,
) {
    let entity_buffer = entity_buffers.get(voxel_editor.entity).unwrap();
    camera_controller.margins.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            ui.label(format!("Entity: {:?}", voxel_editor.entity));
            ui.label(format!("Prefab Entity: {:?}", voxel_editor.prefab_entity));
            ui.label(format!("Buffer dirty: {}", entity_buffer.buffer_dirty));
            ui.label(format!("Undo stack: {}", entity_buffer.undo_stack.len()));

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

            let prefab = voxel_editor.prefab_entity;
            draw_entity_tree(ui, &mut voxel_editor, prefab, &ui_query);
        })
        .response
        .rect
        .width();
}

fn draw_entity_tree(
    ui: &mut Ui,
    voxel_editor: &mut EditorResource,
    entity: Entity,
    query: &Query<(&Name, Option<&Children>, Option<&EntityBuffer>)>,
) {
    let (name, children, entity_buffer) = query.get(entity).unwrap();
    let name = format!(
        "{}{}",
        name.as_str(),
        if voxel_editor.entity == entity {
            " (active)"
        } else {
            ""
        }
    );

    if let Some(children) = children {
        CollapsingHeader::new(name)
            .default_open(true)
            .show(ui, |ui| {
                if entity_buffer.is_some() && ui.button("Activate").clicked() {
                    voxel_editor.entity = entity;
                }
                for child in children.iter() {
                    draw_entity_tree(ui, voxel_editor, *child, query);
                }
            });
    } else {
        ui.label(name);
        if entity_buffer.is_some() && ui.button("Activate").clicked() {
            voxel_editor.entity = entity;
        }
    }
}
