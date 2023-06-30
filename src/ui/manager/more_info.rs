use eframe::egui::{Align2, Area, CollapsingHeader, Color32, Context, FontFamily, FontId, Frame, Margin, Rect, ScrollArea, Sense, Separator, Stroke, TextStyle, Ui, vec2, Widget};
use egui_toast::Toasts;
use tokio::sync::mpsc::Sender;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use egui_modal::Modal;
use crate::manager::ManagerCommand;
use crate::manifest::{Category, GlobalModList, Mod, ModVersion};
use crate::ui::manager::mod_list::ModEntry;
use crate::ui::manager::UIManagerState;
use crate::utils::{get_next_id, handle_error};
use crate::version::Version;

pub enum MarkdownContent {
    Loading,
    NoReadme,
    Markdown(String)
}

pub struct InfoModalState {
    modal: Modal,
    pub id: Option<String>,
    pub info: Option<Mod>,
    pub versions: Vec<(Version, ModVersion)>,
    pub tab: InfoModalTabs,
    cache: CommonMarkCache,
    pub markdown_content: MarkdownContent
}

impl InfoModalState {
    pub(crate) fn from_context(ctx: &Context) -> Self {
        Self {
            modal: Modal::new(ctx, "more_info_modal")
                .with_close_on_outside_click(true),
            id: None,
            info: None,
            versions: vec![],
            tab: InfoModalTabs::Readme,
            cache: CommonMarkCache::default(),
            markdown_content: MarkdownContent::Loading,
        }
    }

    fn fill_in_info(&mut self, mod_entry: &ModEntry, global_mods: &GlobalModList) {
        self.info = Some(mod_entry.id.clone().and_then(|x| global_mods.mod_list.load()
            .get(&x).cloned()).unwrap_or_else(|| Mod {
            name: mod_entry.name.clone(),
            color: None,
            description: "File that wasn't recognized".to_string(),
            authors: Default::default(),
            source_location: None,
            website: None,
            tags: None,
            category: Category::Unknown,
            flags: None,
            versions: Default::default(),
        }));
        self.id = mod_entry.id.clone();

        self.versions.clear();

        if let Some(mod_info) = &self.info {
            self.versions.extend(
                mod_info.versions.iter()
                .map(|(v, i)| (v.clone(), i.clone()))
            );

            self.versions.sort_by(|(a_v, _), (b_v, _)| {
                b_v.cmp(a_v)
            });
        }
    }

    pub(crate) fn open_with_entry_data(&mut self, mod_entry: &ModEntry, global_mods: &GlobalModList, toasts: &mut Toasts, command: &Sender<ManagerCommand>) {
        self.fill_in_info(mod_entry, global_mods);
        self.tab = InfoModalTabs::Readme;
        self.markdown_content = MarkdownContent::Loading;
        self.modal.open();

        match &mod_entry.id {
            Some(guid) => {
                handle_error(command.blocking_send(ManagerCommand::FindReadmeFor(guid.clone())), toasts);
            }
            None => {
                self.markdown_content = MarkdownContent::NoReadme
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InfoModalTabs {
    Readme,
    Versions
}

pub fn more_info_modal(state: &mut UIManagerState, ctx: &Context, toasts: &mut Toasts, command: &Sender<ManagerCommand>) {
    let info_modal_state = &mut state.mod_list_state.more_info;

    info_modal_state.modal.show(|ui| {
        let pos = ui.next_widget_position();
        ui.expand_to_include_rect(Rect::from_min_size(pos, vec2(750.0, 600.0)));

        if let Some(mod_info) = &info_modal_state.info {
            match more_info_header(ui, mod_info, &info_modal_state.id, &info_modal_state.tab) {
                MoreInfoHeaderResponse::Nothing => {}
                MoreInfoHeaderResponse::CloseRequested => {
                    info_modal_state.modal.close();
                }
                MoreInfoHeaderResponse::ChangeTab(new_tab) => {
                    info_modal_state.tab = new_tab;
                }
                MoreInfoHeaderResponse::OpenWebsite => {
                    handle_error(open::that(mod_info.website.as_ref().unwrap()), toasts);
                }
                MoreInfoHeaderResponse::OpenSource => {
                    handle_error(open::that(mod_info.source_location.as_ref().unwrap()), toasts);
                }
            }

            match info_modal_state.tab {
                InfoModalTabs::Readme => {
                    Frame::default()
                        .outer_margin(Margin {
                            left: 0.0,
                            right: 0.0,
                            top: 5.0,
                            bottom: 0.0,
                        })
                        .show(ui, |ui| {
                            match &info_modal_state.markdown_content {
                                MarkdownContent::Loading => {
                                    ui.centered_and_justified(|ui| {
                                        ui.heading("Loading README...")
                                    });
                                }
                                MarkdownContent::NoReadme => {
                                    ui.centered_and_justified(|ui| {
                                        ui.heading("No README file")
                                    });
                                }
                                MarkdownContent::Markdown(md) => {
                                    ScrollArea::vertical()
                                        .id_source("more_info_readme_scroll")
                                        .auto_shrink([false; 2])
                                        .max_height(500.0)
                                        .show(ui, |ui| {
                                            CommonMarkViewer::new("more_info_readme")
                                                .max_image_width(Some(700))
                                                .show(ui, &mut info_modal_state.cache, md);
                                        });
                                }
                            }
                        });
                }

                InfoModalTabs::Versions => {
                    Frame::default()
                        .outer_margin(Margin {
                            left: 0.0,
                            right: 0.0,
                            top: 5.0,
                            bottom: 0.0,
                        })
                        .show(ui, |ui| {
                            if info_modal_state.versions.len() > 0 {
                                ScrollArea::vertical()
                                    .id_source("more_info_version_scroll")
                                    .auto_shrink([false; 2])
                                    .max_height(500.0)
                                    .show(ui, |ui| {
                                        for (version, version_info) in &info_modal_state.versions {
                                            more_info_version(ui, version, version_info);
                                        }
                                    });
                            } else {
                                ui.centered_and_justified(|ui| {
                                    ui.heading("No version info");
                                });
                            }
                        });
                }
            }
        }
    });
}

fn more_info_version(ui: &mut Ui, version: &Version, version_info: &ModVersion) {
    Frame::default()
        .fill(ui.visuals().widgets.inactive.bg_fill)
        .outer_margin(5.0)
        .inner_margin(10.0)
        .rounding(4.0)
        .show(ui, |ui| {
            let pos = ui.next_widget_position();
            ui.expand_to_include_rect(Rect::from_min_size(pos, vec2(ui.max_rect().width(), 20.0)));

            ui.heading(format!("v{}", version));

            if let Some(changelog) = &version_info.changelog {
                ui.label(changelog);
            } else {
                ui.small("- Empty changelog -");
            }

            if version_info.dependencies.is_some() || version_info.conflicts.is_some() {
                ui.scope(|ui| {
                    ui.style_mut().visuals.widgets.noninteractive.bg_stroke.color = Color32::from_rgba_premultiplied(100, 100, 100, 255);

                    ui.separator();
                });
            }

            if let Some(dependencies) = &version_info.dependencies {
                CollapsingHeader::new("Dependencies")
                    .id_source(get_next_id(ui))
                    .show(ui, |ui| {
                        for (guid, dependency) in dependencies {
                            ui.label(format!("• {} {}", guid, dependency.version));
                        }
                    });
            }

            if let Some(conflicts) = &version_info.conflicts {
                CollapsingHeader::new("Conflicts")
                    .id_source(get_next_id(ui))
                    .show(ui, |ui| {
                        for (guid, conflict) in conflicts {
                            ui.label(format!("• {} {}", guid, conflict.version));
                        }
                    });
            }

        });
}

enum MoreInfoHeaderResponse {
    Nothing,
    CloseRequested,
    ChangeTab(InfoModalTabs),
    OpenWebsite,
    OpenSource,
}

fn more_info_header(ui: &mut Ui, mod_info: &Mod, id: &Option<String>, current_tab: &InfoModalTabs) -> MoreInfoHeaderResponse {
    let normal_text = ui.style().text_styles.get(&TextStyle::Body).cloned().unwrap_or_else(|| FontId { size: 15.0, family: FontFamily::Proportional });
    let small_text = ui.style().text_styles.get(&TextStyle::Small).cloned().unwrap_or_else(|| FontId { size: 12.0, family: FontFamily::Proportional });
    let icon_id = FontId { size: 20.0, family: FontFamily::Proportional };

    let tabs_visuals = ui.visuals().widgets.inactive;
    let header_visuals = ui.visuals().widgets.hovered;

    let element_width = ui.max_rect().width().max(750.0);

    let header_height = 80.0_f32;
    let tabs_height = 35.0_f32;
    let total_height = header_height + tabs_height;

    let tabs_width = 100.0_f32;
    let tabs_gap = 4.0_f32;

    let close_button_size = 60.0_f32;
    let close_button_font_id = FontId::new(38.0, FontFamily::Proportional);

    // Allocating space for the header
    let (element_rect, _) = ui.allocate_exact_size(vec2(element_width, total_height), Sense::hover());

    // Painting background
    ui.painter().rect(element_rect, 4.0, tabs_visuals.bg_fill, Stroke::NONE);

    // Header part
    let header_rect = Rect::from_min_size(element_rect.left_top(), vec2(element_width, header_height));
    let header_size = header_rect.size();

    ui.painter().rect(header_rect, 4.0, header_visuals.bg_fill, Stroke::NONE);

    // Close button
    let close_button_offset = (header_size.y / 2.0 - close_button_size / 2.0);
    let close_button_rect = Rect {
        min: header_rect.right_top() + vec2(-header_height + close_button_offset, close_button_offset),
        max: header_rect.right_bottom() - vec2(close_button_offset, close_button_offset),
    };

    let close_button_id = get_next_id(ui);
    let close_button_response = ui.interact(close_button_rect, close_button_id, Sense::click());

    let close_button_visuals = ui.style().interact(&close_button_response);

    ui.painter().rect(close_button_rect.expand(close_button_visuals.expansion), 4.0, close_button_visuals.bg_fill, close_button_visuals.bg_stroke);
    ui.painter().text(close_button_rect.center(), Align2::CENTER_CENTER, "✖", close_button_font_id, close_button_visuals.text_color());

    // The actual text of the header
    let text_container = Rect::from_min_size(element_rect.left_top(), vec2(element_width - header_height, header_height));
    let actual_text_bounds = text_container.shrink(5.0);
    let wrap_width = actual_text_bounds.width() - 10.0;

    let title_galley = ui.painter()
        .layout(mod_info.name.clone(), normal_text.clone(), Color32::BLACK, wrap_width);
    let id_galley = id.as_ref().map(|x| {
        ui.painter().layout(x.clone(), small_text.clone(), Color32::BLACK, wrap_width)
    });

    let title_height = title_galley.rect.height();
    let id_height = id_galley.as_ref().map_or(0.0_f32, |x| x.rect.height());

    let total_text_height = title_height + 3.0 + id_height;

    let text_start_position = actual_text_bounds.left_center() - vec2(-10.0, total_text_height / 2.0 - 1.5);

    let text_painter = ui.painter_at(actual_text_bounds);
    text_painter.galley_with_color(text_start_position, title_galley, header_visuals.text_color());

    if let Some(id_galley) = id_galley {
        text_painter.galley_with_color(text_start_position + vec2(0.0, title_height + 3.0), id_galley, Color32::GRAY);
    }

    // Tab buttons
    let tab_buttons = [
        ("README", InfoModalTabs::Readme),
        ("Versions", InfoModalTabs::Versions)
    ];

    let mut offset = tabs_gap;
    for (tab_name, tab_value) in tab_buttons {
        let tab_selected = *current_tab == tab_value;
        let tab_id = get_next_id(ui);

        let tab_start_pos = element_rect.left_bottom() + vec2(offset, -tabs_height);
        let tab_rect = Rect::from_min_size(tab_start_pos, vec2(tabs_width, tabs_height))
            .shrink2(vec2(0.0, tabs_gap));

        offset += tabs_width + tabs_gap;

        let tab_response = ui.interact(tab_rect, tab_id, Sense::click());

        let tab_visuals = ui.style().interact_selectable(&tab_response, tab_selected);

        ui.painter().rect(tab_rect, 4.0, tab_visuals.bg_fill, tab_visuals.bg_stroke);
        ui.painter().text(tab_rect.center(), Align2::CENTER_CENTER, tab_name, normal_text.clone(), tab_visuals.text_color());

        if tab_response.clicked() {
            return MoreInfoHeaderResponse::ChangeTab(tab_value);
        }
    }

    let site_buttons = [
        if mod_info.website.is_some() { Some(("🌐", MoreInfoHeaderResponse::OpenWebsite)) } else { None },
        if mod_info.source_location.is_some() { Some(("", MoreInfoHeaderResponse::OpenSource)) } else { None }
    ];

    let mut offset = element_width - tabs_height;
    for button in site_buttons {
        let tab_id = get_next_id(ui);

        if let Some((tab_label, action)) = button {
            let tab_start_pos = element_rect.left_bottom() + vec2(offset, -tabs_height);
            let tab_rect = Rect::from_min_size(tab_start_pos, vec2(tabs_height, tabs_height))
                .shrink(tabs_gap);

            offset -= tabs_height;

            let tab_response = ui.interact(tab_rect, tab_id, Sense::click());

            let tab_visuals = ui.style().interact(&tab_response);

            ui.painter().rect(tab_rect, 4.0, tab_visuals.bg_fill, tab_visuals.bg_stroke);
            ui.painter().text(tab_rect.center(), Align2::CENTER_CENTER, tab_label, icon_id.clone(), tab_visuals.text_color());

            if tab_response.clicked() {
                return action;
            }
        }
    }

    if close_button_response.clicked() {
        return MoreInfoHeaderResponse::CloseRequested;
    }

    MoreInfoHeaderResponse::Nothing
}
