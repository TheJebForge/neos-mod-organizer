use std::cmp::max;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use arc_swap::ArcSwap;
use eframe::egui::{Align2, Area, Color32, ComboBox, Context, FontFamily, FontId, Frame, Margin, Pos2, pos2, Rect, Resize, Response, RichText, ScrollArea, Sense, Stroke, TextEdit, TextFormat, TextStyle, Ui, Vec2, vec2, Widget};
use eframe::egui::text::LayoutJob;
use eframe::epaint::text::TextWrapping;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use egui_modal::Modal;
use egui_toast::Toasts;
use futures::StreamExt;
use tokio::sync::mpsc::Sender;
use crate::config::Config;
use crate::install::ModMap;
use crate::manager::ManagerCommand;
use crate::manifest::{Category, GlobalModList, Mod};
use crate::ui::manager::more_info::InfoModalState;
use crate::ui::manager::UIManagerState;
use crate::utils::{get_next_id, handle_error, lerp_color, lerp_f32};
use crate::version::Version;

pub struct ModListState {
    mod_view: ModView,
    filter: String,
    last_mod_count: usize,
    expanded_entry: u64,
    pub more_info: InfoModalState
}

impl ModListState {
    pub fn from_context(ctx: &Context) -> Self {
        Self {
            mod_view: Default::default(),
            filter: "".to_string(),
            last_mod_count: 0,
            expanded_entry: 0,
            more_info: InfoModalState::from_context(ctx),
        }
    }
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

#[derive(Debug, Hash)]
pub struct ModEntry {
    category: Category,
    pub(crate) name: String,
    pub(crate) id: Option<String>,
    version: Option<Version>,
    latest_version: Option<Version>,
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

    ScrollArea::vertical()
        .show(ui, |ui| {
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

                                let mut first_one = true;

                                for mod_item in category_mods {
                                    let mut hasher = DefaultHasher::new();
                                    mod_item.hash(&mut hasher);
                                    let hash = hasher.finish();

                                    match draw_mod_entry(ui, mod_item, first_one, mod_list_state.expanded_entry == hash) {
                                        DrawModEntryResponse::Nothing => {}
                                        DrawModEntryResponse::ToggleExpand => {
                                            if mod_list_state.expanded_entry == hash {
                                                mod_list_state.expanded_entry = 0;
                                            } else {
                                                mod_list_state.expanded_entry = hash;
                                            }
                                        }
                                        DrawModEntryResponse::ToggleEnabled => {
                                            mod_item.enabled = !mod_item.enabled;
                                        }
                                        DrawModEntryResponse::MoreInfo => {
                                            mod_list_state.more_info.open_with_data(mod_item, global_mods, toasts, command);
                                        }
                                        DrawModEntryResponse::Uninstall => {}
                                        DrawModEntryResponse::Update => {}
                                    }

                                    first_one = false;
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
                            let mut first_one = true;

                            for mod_item in mods {
                                let mut hasher = DefaultHasher::new();
                                mod_item.hash(&mut hasher);
                                let hash = hasher.finish();

                                match draw_mod_entry(ui, mod_item, first_one, mod_list_state.expanded_entry == hash) {
                                    DrawModEntryResponse::Nothing => {}
                                    DrawModEntryResponse::ToggleExpand => {
                                        if mod_list_state.expanded_entry == hash {
                                            mod_list_state.expanded_entry = 0;
                                        } else {
                                            mod_list_state.expanded_entry = hash;
                                        }
                                    }
                                    DrawModEntryResponse::ToggleEnabled => {
                                        mod_item.enabled = !mod_item.enabled;
                                    }
                                    DrawModEntryResponse::MoreInfo => {
                                        mod_list_state.more_info.open_with_data(mod_item, global_mods, toasts, command);
                                    }
                                    DrawModEntryResponse::Uninstall => {}
                                    DrawModEntryResponse::Update => {}
                                }

                                first_one = false;
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
        });
}

fn draw_mod_entry(ui: &mut Ui, entry: &ModEntry, first_one: bool, expanded: bool) -> DrawModEntryResponse {
    let inter_mod_gap = 10_f32;

    // Prefix
    let target_prefix = if expanded && !first_one {
        inter_mod_gap
    } else {
        0.0
    };

    let animated_prefix_id = get_next_id(ui);
    let animated_prefix = ui.ctx().animate_value_with_time(animated_prefix_id, target_prefix, 0.1);

    ui.add_space(animated_prefix);

    // The actual component starts here
    let normal_text = ui.style().text_styles.get(&TextStyle::Body).cloned().unwrap_or_else(|| FontId { size: 15.0, family: FontFamily::Proportional });
    let small_text = ui.style().text_styles.get(&TextStyle::Small).cloned().unwrap_or_else(|| FontId { size: 12.0, family: FontFamily::Proportional });

    let max_rect = ui.max_rect();
    let element_width = max_rect.width();
    let element_left_top = ui.next_widget_position();
    let element_height = 60.0_f32;

    let button_width = 100_f32;
    let button_height = 20_f32;

    // Arrow calculations
    let arrow_font_id = FontId { size: 25.0, family: FontFamily::Proportional };
    let arrow_width = 30.0_f32;
    let arrow_point = element_left_top + vec2(element_width - arrow_width / 2.0 - 10.0, element_height / 2.0 + if expanded {
        1.0
    } else {
        0.0
    });

    // Checkbox calculation
    const CHECKBOX_MAX_SIZE: f32 = 30.0;

    let checkbox_offset = element_height / 2.0 - CHECKBOX_MAX_SIZE / 2.0;
    let checkbox_starting_pos = element_left_top + Vec2::new(element_width - arrow_width - element_height + checkbox_offset, checkbox_offset);
    let checkbox_end_pos = element_left_top + Vec2::new(element_width - arrow_width - checkbox_offset, element_height - checkbox_offset);
    let checkbox_rect = Rect::from([checkbox_starting_pos, checkbox_end_pos]);

    // Text container
    let text_container_width = element_width - element_height - arrow_width;

    // Expand calculations
    let mut description_galley = if expanded {
        entry.description.as_ref().map(|x| {
            ui.painter().layout(x.clone(), small_text.clone(), Color32::BLACK, element_width - 20.0)
        })
    } else {
        None
    };

    let target_height = if expanded {
        if let Some(galley) = &description_galley {
            galley.rect.height() + button_height + 25.0 + inter_mod_gap
        } else {
            button_height + 10.0 + inter_mod_gap
        }
    } else {
        0.0
    };

    let animated_spacer = ui.ctx().animate_value_with_time(ui.next_auto_id(), target_height, 0.1);
    let expanded_rect = Rect::from_min_size(element_left_top, vec2(element_width, element_height + animated_spacer - inter_mod_gap));

    // Responses
    let (element_rect, mut element_response) = ui.allocate_exact_size(Vec2::new(element_width, element_height), Sense::click());
    let checkbox_id = get_next_id(ui);
    let mut checkbox_response = ui.interact(checkbox_rect.clone(), checkbox_id, Sense::click());

    let more_info_id = get_next_id(ui);
    let uninstall_id = get_next_id(ui);
    let update_id = get_next_id(ui);

    let mut additional_responses = if animated_spacer > 0.1 {
        let more_info_pos = expanded_rect.right_bottom() - vec2(5.0 + button_width, 5.0 + button_height);
        let uninstall_pos = more_info_pos - vec2(5.0 + button_width, 0.0);
        let update_pos = uninstall_pos - vec2(5.0 + button_width, 0.0);

        let more_info_rect = Rect::from_min_size(more_info_pos, vec2(button_width, button_height));
        let uninstall_rect = Rect::from_min_size(uninstall_pos, vec2(button_width, button_height));
        let update_rect = Rect::from_min_size(update_pos, vec2(button_width, button_height));

        Some((
            ui.interact(more_info_rect, more_info_id, Sense::click()),
            ui.interact(uninstall_rect, uninstall_id, Sense::click()),
            ui.interact(update_rect, update_id, Sense::click()),
        ))
    } else {
        None
    };

    ui.add_space(animated_spacer);

    // Actually painting
    if ui.is_rect_visible(element_rect) {
        // Latest version test
        let is_latest = entry.version.as_ref().and_then(|x| {
            let latest = entry.latest_version.as_ref()?;
            Some(x >= latest)
        }).unwrap_or(true);

        // Fixing title text
        let no_new_line_name = entry.name.replace('\n', "\\n");

        let title = if no_new_line_name.len() > 80 {
            format!("{}...", no_new_line_name.chars().take(80).collect::<String>())
        } else {
            no_new_line_name
        };

        // Calculating text sizes
        let title_galley = ui.painter().layout(
            title,
            normal_text.clone(),
            Color32::BLACK,
            text_container_width
        );

        let id_version_text = entry.id.as_ref().map(|x| {
            let id = x.replace('\n', "\\n");

            let id = if id.len() > 55 {
                format!("{}...", id.chars().take(55).collect::<String>())
            } else {
                id
            };

            let mut job = LayoutJob {
                wrap: TextWrapping {
                    max_width: text_container_width,
                    ..Default::default()
                },
                ..Default::default()
            };

            job.append(&format!("{} ", id), 0.0, TextFormat {
                font_id: small_text.clone(),
                color: Color32::GRAY,
                ..Default::default()
            });

            if is_latest {
                if let Some(version) = &entry.version {
                    job.append(&format!("v{}", version), 0.0, TextFormat {
                        font_id: small_text.clone(),
                        color: Color32::GRAY,
                        ..Default::default()
                    });
                }
            } else {
                if let Some(version) = &entry.version {
                    job.append(&format!("v{}", version), 0.0, TextFormat {
                        font_id: small_text.clone(),
                        color: Color32::LIGHT_RED,
                        ..Default::default()
                    });

                    job.append(", ", 0.0, TextFormat {
                        font_id: small_text.clone(),
                        color: Color32::GRAY,
                        ..Default::default()
                    });

                    if let Some(latest_version) = &entry.latest_version {
                        job.append(&format!("latest is v{}", latest_version), 0.0, TextFormat {
                            font_id: small_text.clone(),
                            color: Color32::LIGHT_GREEN,
                            ..Default::default()
                        });
                    }
                }
            }

            job
        });

        let id_galley = id_version_text.map(|x| ui.ctx().fonts(|f| f.layout_job(x)));

        let title_height = title_galley.rect.height();
        let id_height = id_galley.as_ref().map_or(0.0, |x| x.rect.height());

        // Drawing the main element background
        let element_visuals = if (element_response.is_pointer_button_down_on() || element_response.has_focus()) && !checkbox_response.is_pointer_button_down_on() {
            ui.style().visuals.widgets.active
        } else if element_response.hovered() || element_response.highlighted() {
            ui.style().visuals.widgets.hovered
        } else {
            ui.style().visuals.widgets.inactive
        };

        let bg_rect = expanded_rect.clone();
        let fg_rect = element_rect.expand(element_visuals.expansion);

        ui.painter()
            .rect(bg_rect, 4.0, ui.visuals().widgets.noninteractive.bg_fill, ui.visuals().widgets.noninteractive.bg_stroke);

        // Drawing additional options here
        if let Some((ref mut more_info, ref mut uninstall, ref mut update)) = &mut additional_responses {
            let element_bottom_pos = expanded_rect.left_bottom();

            if let Some(description_galley) = description_galley {
                let description_pos = element_bottom_pos + vec2(10.0, -13.0 - button_height - description_galley.rect.height());
                ui.painter().galley_with_color(
                    description_pos,
                    description_galley,
                    Color32::LIGHT_GRAY
                );
            }

            draw_button(ui, "More Info", normal_text.clone(), more_info, true);
            draw_button(ui, "Uninstall", normal_text.clone(), uninstall, true);
            draw_button(ui, "Update", normal_text.clone(), update, !is_latest);
        }

        // Drawing the mod button
        ui.painter()
            .rect(fg_rect, 4.0, element_visuals.bg_fill, element_visuals.bg_stroke);

        // Drawing the arrow
        ui.painter().text(arrow_point, Align2::CENTER_CENTER, if expanded {
            format!("⏷")
        } else {
            format!("⏵")
        }, arrow_font_id.clone(), element_visuals.text_color());

        // Drawing the checkbox
        let checkbox_selected_visuals = ui.style().interact_selectable(&checkbox_response, true);
        let checkbox_visuals = ui.style().interact_selectable(&checkbox_response, entry.enabled);

        let target = if entry.enabled { 1.0 } else { 0.0 };

        let t = ui.ctx().animate_value_with_time(checkbox_id, target, 0.2);
        let lerped_color = lerp_color(&ui.style().visuals.panel_fill, &checkbox_selected_visuals.bg_fill, t);
        let lerped_transparency_color = lerp_color(&Color32::TRANSPARENT, &checkbox_selected_visuals.bg_fill, t);

        let checkbox_shrink = lerp_f32(4.0, 0.0, t);
        let checkbox_current_rect = checkbox_rect.shrink(checkbox_shrink);

        let inner_box_size = lerp_f32(3.0, 23.0, t);
        let inner_box_rect = Rect::from_center_size(checkbox_current_rect.center(), vec2(inner_box_size, inner_box_size));

        ui.painter().rect(
            checkbox_current_rect.expand(checkbox_visuals.expansion + 1.0),
            4.0,
            Color32::TRANSPARENT,
            checkbox_visuals.bg_stroke
        );

        ui.painter().rect(
            checkbox_current_rect,
            2.0,
            Color32::TRANSPARENT,
            Stroke::new(3.0, lerped_color)
        );

        ui.painter().rect(
            inner_box_rect,
            2.0,
            lerped_transparency_color,
            Stroke::new(0.0, Color32::TRANSPARENT)
        );

        // Calculating text position
        let text_height = 2.0 + title_height + id_height;

        let title_pos = element_left_top + vec2(10.0, element_height / 2.0 - text_height / 2.0);
        let id_pos = title_pos + vec2(0.0, title_height + 2.0);

        // Drawing text in separate clipped painter
        let text_painter = ui.painter_at(Rect::from_min_size(element_left_top.clone(), vec2(text_container_width, element_height)));

        text_painter.galley_with_color(
            title_pos,
            title_galley,
            element_visuals.text_color(),
        );

        if let Some(id_galley) = id_galley {
            text_painter.galley(
                id_pos,
                id_galley
            );
        }
    }

    if let Some((more_info, uninstall, update)) = additional_responses {
        if more_info.clicked() {
            return DrawModEntryResponse::MoreInfo;
        } else if uninstall.clicked() {
            return DrawModEntryResponse::Uninstall;
        } else if update.clicked() {
            return DrawModEntryResponse::Update;
        }
    }

    if checkbox_response.clicked() {
        DrawModEntryResponse::ToggleEnabled
    } else if element_response.clicked() {
        DrawModEntryResponse::ToggleExpand
    } else {
        DrawModEntryResponse::Nothing
    }
}

fn draw_button(ui: &mut Ui, text: &str, font_id: FontId, mut response: &mut Response, enabled: bool) {
    let rect = response.rect;

    let visuals = if enabled {
        ui.style().interact(&response)
    } else {
        &ui.style().visuals.widgets.noninteractive
    };

    // Button background
    ui.painter()
        .rect(rect.clone(), 4.0, visuals.bg_fill, visuals.bg_stroke);

    // Text
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, text, font_id, visuals.text_color());
}

enum DrawModEntryResponse {
    Nothing,
    ToggleExpand,
    ToggleEnabled,
    MoreInfo,
    Uninstall,
    Update
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
                latest_version: manifest_mod.versions.iter().map(|(v, _)| v).max().cloned(),
                description: Some(manifest_mod.description.clone()),
                enabled: file.files.iter().all(|x| !x.disabled),
            })
        } else {
            mods.push(ModEntry {
                category: Category::Unknown,
                name: mod_id.clone(),
                id: None,
                version: None,
                latest_version: None,
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