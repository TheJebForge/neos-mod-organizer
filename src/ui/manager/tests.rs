use std::str::FromStr;
use eframe::egui::{CentralPanel, Context, Ui};
use egui_toast::Toasts;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::manager::{ManagerCommand, ManagerEvent};
use crate::ui::manager::UIManagerState;
use crate::version::{Version, VersionReq};

#[derive(Default)]
pub struct TestState {
    a: String,
    b: String,
    result: String,
    req: String,
    req_result: String,
    req_version: String,
    matches: String
}

pub fn test_ui(state: &mut UIManagerState, ui: &mut Ui, toasts: &mut Toasts, command: &Sender<ManagerCommand>, event: &mut Receiver<ManagerEvent>) {
    ui.label("Version comparison test");
    ui.text_edit_singleline(&mut state.test_state.a);
    ui.text_edit_singleline(&mut state.test_state.b);
    if ui.button("compare").clicked() {
        let a = Version::from_str(&state.test_state.a);
        let b = Version::from_str(&state.test_state.b);

        if let Ok(a) = a {
            if let Ok(b) = b {
                state.test_state.result = format!("a: {:#?}\nb: {:#?}\nresult: {:#?}", a, b, a.cmp(&b));
            } else {
                state.test_state.result = format!("a failed to parse");
            }
        } else {
            state.test_state.result = format!("a failed to parse");
        }
    }
    ui.label(&state.test_state.result);

    ui.separator();

    ui.label("Version requirement parse test");
    ui.text_edit_singleline(&mut state.test_state.req);
    if ui.button("parse").clicked() {
        let req = VersionReq::from_str(&state.test_state.req);
        let text = if let Ok(req) = &req {
            req.to_string()
        } else {
            "err".to_string()
        };

        state.test_state.req_result = format!("result: {:#?}\ntext: {}", req, text)
    }
    ui.label(&state.test_state.req_result);

    ui.label("Version to match");
    ui.text_edit_singleline(&mut state.test_state.req_version);
    if ui.button("match").clicked() {
        let text = if let Ok(req) = VersionReq::from_str(&state.test_state.req) {
            if let Ok(ver) = Version::from_str(&state.test_state.req_version) {
                format!("matches: {}", req.matches(&ver))
            } else {
                format!("invalid version")
            }
        } else {
            format!("invalid requirement")
        };

        state.test_state.matches = text;
    }
    ui.label(&state.test_state.matches);
}