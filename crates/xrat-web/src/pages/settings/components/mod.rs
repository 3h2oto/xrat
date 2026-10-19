use crate::ui::{
    button::{Button, ButtonVariant},
    form_select::{FormSelect, FormSelectOption},
    input::{Input, InputType},
    label::Label,
    switch::Switch,
    textarea::Textarea,
    tooltip::{Tooltip, TooltipContent, TooltipPosition},
};
use dioxus::prelude::*;
use icons::{Check, Clock, Cpu, FileCode, FlaskConical, Globe, Route, Rss, Server, Terminal};
use std::collections::BTreeMap;
use xrat_web::types::Setting;

mod setting_row;
pub(crate) use setting_row::*;
mod category_icon;
pub(crate) use category_icon::*;
