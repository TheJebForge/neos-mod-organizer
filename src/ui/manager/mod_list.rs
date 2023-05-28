use std::cmp::max;
use std::collections::HashMap;
use std::sync::Arc;
use arc_swap::ArcSwap;
use eframe::egui::{Align2, Color32, ComboBox, Context, FontFamily, FontId, Pos2, Rect, Sense, Stroke, TextEdit, TextStyle, Ui, Vec2, vec2, Widget};
use egui_toast::Toasts;
use futures::StreamExt;
use tokio::sync::mpsc::Sender;
use crate::config::Config;
use crate::install::ModMap;
use crate::manager::ManagerCommand;
use crate::manifest::{Category, GlobalModList};
use crate::ui::manager::UIManagerState;
use crate::utils::{lerp_color, lerp_f32};
use crate::version::Version;

#[derive(Default)]
pub struct ModListState {
    mod_view: ModView,
    filter: String,
    last_mod_count: usize
}

pub enum ModView {
    NotInitialized,
    Category(Vec<(String, Vec<ModEntry>)>),
    All(Vec<ModEntry>)
}

impl ModView {
    pub fn variant(&self) -> String {
        match self {
            ModView::Category(_) => format!("Category"),
            ModView::NotInitialized | ModView::All(_) => format!("Alphabetic")
        }
    }

    pub fn is_category(&self) -> bool {
        if let ModView::Category(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_all(&self) -> bool {
        if let ModView::All(_) = self {
            true
        } else {
            false
        }
    }
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

    ui.horizontal(|ui| {
        if TextEdit::singleline(&mut state.mod_list_state.filter)
            .hint_text("Search")
            .desired_width(250.0)
            .ui(ui).changed() {
            let mut mods = build_entries(mod_map, global_mods);

            if !state.mod_list_state.filter.is_empty() {
                mods.retain(|x| filter_entry(&state.mod_list_state.filter, x))
            }

            match &state.mod_list_state.mod_view {
                ModView::Category(_) => state.mod_list_state.mod_view = ModView::Category(split_by_categories(mods)),
                ModView::NotInitialized | ModView::All(_) => state.mod_list_state.mod_view = ModView::All(mods)
            }
        }

        ui.separator();

        ComboBox::from_label("Sort by")
            .selected_text(state.mod_list_state.mod_view.variant())
            .width(120.0)
            .show_ui(ui, |ui| {
                { // Category
                    let mut response = ui.selectable_label(state.mod_list_state.mod_view.is_category(), "Category");
                    if response.clicked() && !state.mod_list_state.mod_view.is_category() {
                        state.mod_list_state.last_mod_count = 0;
                        state.mod_list_state.mod_view = ModView::Category(vec![]);
                        response.mark_changed();
                    }
                }

                { // All
                    let mut response = ui.selectable_label(state.mod_list_state.mod_view.is_all(), "Alphabetic");
                    if response.clicked() && !state.mod_list_state.mod_view.is_all() {
                        state.mod_list_state.last_mod_count = 0;
                        state.mod_list_state.mod_view = ModView::All(vec![]);
                        response.mark_changed();
                    }
                }
            });
    });

    ui.separator();

    let mod_list_state = &mut state.mod_list_state;

    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 4.0);

        match &mut mod_list_state.mod_view {
            ModView::NotInitialized => {
                let mut mods = build_entries(mod_map, global_mods);
                mod_list_state.last_mod_count = mods.len();

                if !mod_list_state.filter.is_empty() {
                    mods.retain(|x| filter_entry(&mod_list_state.filter, x))
                }

                mod_list_state.mod_view = ModView::Category(split_by_categories(mods))
            }
            ModView::Category(mods) => {
                if mod_list_state.last_mod_count == mod_map.len() {
                    for (category, category_mods) in mods {
                        ui.heading(category);

                        ui.add_space(2.0);

                        for mod_item in category_mods {
                            match draw_mod_entry(ui, mod_item) {
                                DrawModEntryResponse::Nothing => {}
                                DrawModEntryResponse::OpenModal => {}
                                DrawModEntryResponse::ToggleEnabled => {
                                    mod_item.enabled = !mod_item.enabled;
                                }
                            }
                        }

                        ui.add_space(10.0);
                    }
                } else {
                    let mut mods = build_entries(mod_map, global_mods);
                    mod_list_state.last_mod_count = mods.len();

                    if !mod_list_state.filter.is_empty() {
                        mods.retain(|x| filter_entry(&mod_list_state.filter, x))
                    }

                    mod_list_state.mod_view = ModView::Category(split_by_categories(mods))
                }
            }
            ModView::All(mods) => {
                if mod_list_state.last_mod_count == mod_map.len() {
                    for mod_item in mods {
                        match draw_mod_entry(ui, mod_item) {
                            DrawModEntryResponse::Nothing => {}
                            DrawModEntryResponse::OpenModal => {}
                            DrawModEntryResponse::ToggleEnabled => {
                                mod_item.enabled = !mod_item.enabled;
                            }
                        }
                    }
                } else {
                    let mut mods = build_entries(mod_map, global_mods);
                    mod_list_state.last_mod_count = mods.len();

                    if !mod_list_state.filter.is_empty() {
                        mods.retain(|x| filter_entry(&mod_list_state.filter, x))
                    }

                    mod_list_state.mod_view = ModView::All(mods)
                }
            }
        }
    });
}

fn draw_mod_entry(ui: &mut Ui, entry: &ModEntry) -> DrawModEntryResponse {
    let normal_text = ui.style().text_styles.get(&TextStyle::Body).cloned().unwrap_or_else(|| FontId { size: 15.0, family: FontFamily::Proportional });
    let small_text = ui.style().text_styles.get(&TextStyle::Small).cloned().unwrap_or_else(|| FontId { size: 12.0, family: FontFamily::Proportional });

    let max_rect = ui.max_rect();
    let element_width = max_rect.width();
    let element_left_top = ui.next_widget_position();
    let element_min_height = 60.0_f32;

    let text_container_width = element_width - 60.0;
    let text_wrap_width = text_container_width - 5.0;

    // Calculating text sizes
    let title_galley = ui.painter().layout(
        entry.name.clone(),
        normal_text.clone(),
        Color32::BLACK,
        text_wrap_width
    );

    let id_version_text = entry.id.as_ref().map(|x| {
        if let Some(version) = &entry.version {
            format!("{} v{}", x, version)
        } else {
            format!("{}", x)
        }
    });

    let id_galley = id_version_text.map(|x| ui.painter().layout(
        x,
        small_text.clone(),
        Color32::BLACK,
        text_wrap_width
    ));

    let description_galley = entry.description.clone().map(|x| ui.painter().layout(
        x,
        small_text.clone(),
        Color32::BLACK,
        text_wrap_width
    ));

    let title_height = title_galley.rect.height();
    let id_height = id_galley.as_ref().map_or(0.0, |x| x.rect.height());
    let description_height = description_galley.as_ref().map_or(0.0, |x| x.rect.height());

    let element_height = element_min_height.max(
        title_height + id_height + description_height + 17.5
    );

    // Checkbox calculation
    let checkbox_max_size = 30.0_f32;

    let checkbox_offset = element_min_height / 2.0 - checkbox_max_size / 2.0;
    let checkbox_starting_pos = element_left_top + Vec2::new(element_width - element_min_height + checkbox_offset, checkbox_offset);
    let checkbox_end_pos = element_left_top + Vec2::new(element_width - checkbox_offset, element_min_height - checkbox_offset);
    let checkbox_rect = Rect::from([checkbox_starting_pos, checkbox_end_pos]);

    // Responses
    let (element_rect, mut element_response) = ui.allocate_exact_size(Vec2::new(element_width, element_height), Sense::click());
    let mut checkbox_response = ui.interact(checkbox_rect.clone(), ui.next_auto_id(), Sense::click());

    ui.skip_ahead_auto_ids(1);

    // Actually painting
    if ui.is_rect_visible(element_rect) {
        // Drawing the main element background
        let element_visuals = if (element_response.is_pointer_button_down_on() || element_response.has_focus()) && !checkbox_response.is_pointer_button_down_on() {
            ui.style().visuals.widgets.active
        } else if element_response.hovered() || element_response.highlighted() {
            ui.style().visuals.widgets.hovered
        } else {
            ui.style().visuals.widgets.inactive
        };

        let rect = element_rect.expand(element_visuals.expansion);

        ui.painter()
            .rect(rect, 4.0, element_visuals.bg_fill, element_visuals.bg_stroke);

        // Drawing the checkbox
        let checkbox_selected_visuals = ui.style().interact_selectable(&checkbox_response, true);
        let checkbox_visuals = ui.style().interact_selectable(&checkbox_response, entry.enabled);

        let target = if entry.enabled { 1.0 } else { 0.0 };

        let t = ui.ctx().animate_value_with_time(element_response.id, target, 0.1);
        let lerped_color = lerp_color(&ui.style().visuals.panel_fill, &checkbox_selected_visuals.bg_fill, t);

        let checkbox_shrink = lerp_f32(4.0, 0.0, t);
        let checkbox_current_rect = checkbox_rect.shrink(checkbox_shrink);

        ui.painter().rect(
            checkbox_current_rect,
            4.0,
            lerped_color,
            checkbox_visuals.bg_stroke
        );

        // Drawing text
        ui.painter().galley_with_color(
            element_left_top + Vec2::new(5.0, 5.0),
            title_galley,
            element_visuals.text_color(),
        );

        if let Some(id_galley) = id_galley {
            ui.painter().galley_with_color(
                element_left_top + Vec2::new(5.0, 7.0 + title_height),
                id_galley,
                element_visuals.text_color()
            );

            if let Some(description_galley) = description_galley {
                ui.painter().galley_with_color(
                    element_left_top + Vec2::new(5.0, element_height - description_height - 5.0),
                    description_galley,
                    element_visuals.text_color()
                );
            }
        }
    }

    if checkbox_response.clicked() {
        DrawModEntryResponse::ToggleEnabled
    } else if element_response.clicked() {
        DrawModEntryResponse::OpenModal
    } else {
        DrawModEntryResponse::Nothing
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

fn split_by_categories(entries: Vec<ModEntry>) -> Vec<(String, Vec<ModEntry>)> {
    let mut categories: Vec<(Category, Vec<ModEntry>)> = entries.into_iter()
        .fold(HashMap::new(), |mut map, item| {
            map.entry(item.category)
                .or_insert(vec![])
                .push(item);

            map
        })
        .into_iter()
        .collect();

    // Sort categories
    categories.sort_by(|(a_cat, _), (b_cat, _)| {
        a_cat.cmp(b_cat)
    });

    categories.into_iter()
        .map(|(cat, mods)| (cat.to_string(), mods))
        .collect()
}

fn filter_entry(filter: &str, entry: &ModEntry) -> bool {
    let filter = filter.to_lowercase();

    entry.name.to_lowercase().contains(&filter) ||
        entry.id.as_ref().map_or_else(|| false, |x| x.to_lowercase().contains(&filter)) ||
        entry.description.as_ref().map_or_else(|| false, |x| x.to_lowercase().contains(&filter))
}