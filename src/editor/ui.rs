use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{
    color::Hsva,
    color_picker::{color_picker_hsva_2d, Alpha},
    CollapsingHeader, RichText, Slider, Ui,
};

use crate::{camera::CameraController, voxel::Rgba};

use super::EditorResource;

pub fn editor_ui(
    mut voxel_editor: ResMut<EditorResource>,
    mut egui_context: ResMut<EguiContext>,
    mut camera_controller: ResMut<CameraController>,
    name_children: Query<(&Name, Option<&Children>)>,
) {
    camera_controller.margins.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            ui.label(format!("Entity: {:?}", voxel_editor.entity));
            ui.label(format!("Prefab Entity: {:?}", voxel_editor.prefab_entity));
            ui.label(format!("Buffer dirty: {}", voxel_editor.buffer_dirty));
            ui.label(format!("Undo stack: {}", voxel_editor.undo_stack.len()));

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

            draw_entity_tree(ui, voxel_editor.prefab_entity, &name_children);
        })
        .response
        .rect
        .width();
}

fn draw_entity_tree(
    ui: &mut Ui,
    entity: Entity,
    name_children: &Query<(&Name, Option<&Children>)>,
) {
    let (name, children) = name_children.get(entity).unwrap();

    if let Some(children) = children {
        CollapsingHeader::new(name.as_str())
            .default_open(true)
            .show(ui, |ui| {
                for child in children.iter() {
                    draw_entity_tree(ui, *child, name_children);
                }
            });
    } else {
        ui.label(name.as_str());
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Action {
    Keep,
    Delete,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Tree(Vec<Tree>);

impl Tree {
    pub fn demo() -> Self {
        Self(vec![
            Tree(vec![Tree::default(); 4]),
            Tree(vec![Tree(vec![Tree::default(); 2]); 3]),
        ])
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Action {
        self.ui_impl(ui, 0, "root")
    }
}

impl Tree {
    fn ui_impl(&mut self, ui: &mut Ui, depth: usize, name: &str) -> Action {
        CollapsingHeader::new(name)
            .default_open(depth < 1)
            .show(ui, |ui| self.children_ui(ui, depth))
            .body_returned
            .unwrap_or(Action::Keep)
    }

    fn children_ui(&mut self, ui: &mut Ui, depth: usize) -> Action {
        if depth > 0
            && ui
                .button(RichText::new("delete").color(ui.visuals().warn_fg_color))
                .clicked()
        {
            return Action::Delete;
        }

        self.0 = std::mem::take(self)
            .0
            .into_iter()
            .enumerate()
            .filter_map(|(i, mut tree)| {
                if tree.ui_impl(ui, depth + 1, &format!("child #{}", i)) == Action::Keep {
                    Some(tree)
                } else {
                    None
                }
            })
            .collect();

        if ui.button("+").clicked() {
            self.0.push(Tree::default());
        }

        Action::Keep
    }
}
