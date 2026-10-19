pub(super) mod data_tables;
use crate::api::actions::run;
use crate::ui::icon_button::IconButton;
use crate::{Session, api, ui::button::ButtonVariant};
use dioxus::prelude::*;
use icons::{Play, RotateCcw};
use xrat_web::types::Action;

mod config_actions;
pub(crate) use config_actions::*;
mod config_toggle;
pub(crate) use config_toggle::*;
