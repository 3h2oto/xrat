use crate::ui::icon_button::IconButton;
use crate::ui::{
    button::ButtonVariant,
    dialog::*,
    field::Field,
    form_select::{FormSelect, FormSelectOption},
    input::{Input, InputType},
    label::Label,
    spinner::Spinner,
    switch::Switch,
};
use dioxus::prelude::*;
use icons::{Eye, EyeOff, KeyRound, Plus, Save};
use xrat_web::types::{Role, SaveUserRequest, User};

mod account_dialog;
pub(crate) use account_dialog::*;
