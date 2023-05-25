use std::sync::Arc;
use arc_swap::ArcSwap;
use eframe::egui::{Align2, Color32, Context, Pos2, Rect, Sense, Stroke, TextStyle, Ui, Vec2};
use egui_toast::Toasts;
use tokio::sync::mpsc::Sender;
use crate::config::Config;
use crate::install::ModMap;
use crate::manager::ManagerCommand;
use crate::manifest::{Category, GlobalModList};
use crate::ui::manager::UIManagerState;
use crate::utils::lerp_color;
use crate::version::Version;

#[derive(Default)]
pub struct ModListState {
    mod_view: ModView,
}

pub enum ModView {
    NotInitialized,
    Category(Vec<(String, Vec<ModEntry>)>),
    All(Vec<ModEntry>)
}

impl Default for ModView {
    fn default() -> Self {
        Self::NotInitialized
    }
}

#[derive(Debug)]
pub struct ModEntry {
    category: Category,
    name: String,
    id: Option<String>,
    version: Option<Version>,
    description: Option<String>,
    enabled: bool
}

pub fn mod_list_ui(state: &mut UIManagerState, config: &Arc<ArcSwap<Config>>, ui: &mut Ui, ctx: &Context, toasts: &mut Toasts, command: &Sender<ManagerCommand>) {
    let mod_map = &state.mod_list;
    let global_mods = &state.manifest_mods;

    let mod_list_state = &mut state.mod_list_state;

    match &mut mod_list_state.mod_view {
        ModView::NotInitialized => {
            mod_list_state.mod_view = ModView::All(build_entries(mod_map, global_mods))
        }
        ModView::Category(_) => {}
        ModView::All(mods) => {
            for mod_item in mods {
                match draw_mod_entry(ui, mod_item) {
                    DrawModEntryResponse::Nothing => {}
                    DrawModEntryResponse::OpenModal => {}
                    DrawModEntryResponse::ToggleEnabled => {
                        mod_item.enabled = !mod_item.enabled;
                    }
                }
            }
        }
    }
}

fn draw_mod_entry(ui: &mut Ui, entry: &ModEntry) -> DrawModEntryResponse {
    let element_response = DrawModEntryResponse::Nothing;

    let max_rect = ui.max_rect();

    let (element_rect, mut response) = ui.allocate_exact_size(Vec2::new(max_rect.width(), 60.0), Sense::click());

    if ui.is_rect_visible(element_rect) {
        let visuals = ui.style().interact_selectable(&response, false);

        // Drawing the main element background
        let rect = element_rect.expand(visuals.expansion);

        ui.painter()
            .rect(rect, 4.0, visuals.bg_fill, visuals.bg_stroke);

        // Drawing the checkbox
        let selected_color = ui.style().visuals.selection.bg_fill;

        let target = if entry.enabled { 1.0 } else { 0.0 };

        let t = ui.ctx().animate_value_with_time(response.id, target, 0.1);
        let lerped_color = lerp_color(&visuals.bg_fill, &selected_color, t);

        let rect_width = rect.width();
        let rect_height = rect.height();

        let checkbox_size = 15.0 * (t + 1.0);

        let checkbox_rect = Rect::from([
            rect.left_top() + Vec2::new(
                rect_width - (rect_height / 2.0 + checkbox_size / 2.0),
                (rect_height / 2.0 - checkbox_size / 2.0)
            ),
            rect.left_top() + Vec2::new(
                rect_width - (rect_height / 2.0 - checkbox_size / 2.0),
                (rect_height / 2.0 + checkbox_size / 2.0)
            )
        ]);

        ui.painter()
            .rect(checkbox_rect, 4.0, lerped_color, Stroke::new(0.9, ui.style().visuals.widgets.noninteractive.bg_fill));

        if let Some(font_id) = ui.style().text_styles.get(&TextStyle::Body) {
            ui.painter()
                .text(rect.left_top() + Vec2::new(5.0, 5.0), Align2::LEFT_TOP, &entry.name, font_id.clone(), visuals.text_color());


        }

        if let Some(font_id) = ui.style().text_styles.get(&TextStyle::Small) {
            if let Some(id) = &entry.id {
                ui.painter()
                    .text(rect.left_top() + Vec2::new(5.0, 23.0), Align2::LEFT_TOP,
                          if let Some(version) = &entry.version {
                              format!("{} v{}", id, version)
                          } else {
                              format!("{}", id)
                          }, font_id.clone(), ui.style().visuals.widgets.noninteractive.fg_stroke.color);
            }
        }

    }

    if response.clicked() {
        DrawModEntryResponse::ToggleEnabled
    } else {
        element_response
    }
}

enum DrawModEntryResponse {
    Nothing,
    OpenModal,
    ToggleEnabled
}

fn build_entries(mod_map: &ModMap, global_mods: &GlobalModList) -> Vec<ModEntry> {
    let mut mod_iter = mod_map.iter()
        .filter(|(_, l)| l.len() > 0);

    let global_modlist = global_mods.mod_list.load();
    let mut mods = vec![];

    while let Some((mod_id, versions)) = mod_iter.next() {
        let (version, file) = versions.iter().next().unwrap();

        println!("mod id: {}", mod_id);

        if let Some(manifest_mod) = global_modlist.get(mod_id) {
            mods.push(ModEntry {
                category: manifest_mod.category,
                name: manifest_mod.name.clone(),
                id: Some(mod_id.to_string()),
                version: Some(version.clone()),
                description: Some(manifest_mod.description.clone()),
                enabled: file.files.iter().all(|x| !x.disabled),
            })
        } else {
            mods.push(ModEntry {
                category: Category::Unknown,
                name: mod_id.clone(),
                id: None,
                version: None,
                description: None,
                enabled: file.files.iter().all(|x| !x.disabled),
            })
        }
    }

    mods.sort_by(|a, b| {
        a.name.cmp(&b.name)
    });

    mods
}