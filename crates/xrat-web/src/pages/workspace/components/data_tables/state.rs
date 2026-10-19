use dioxus::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

#[derive(Clone, Copy, PartialEq)]
pub struct Column {
    pub key: &'static str,
    pub label: &'static str,
}
pub const CONFIG_COLUMNS: &[Column] = &[
    Column {
        key: "protocol",
        label: "Protocol",
    },
    Column {
        key: "address",
        label: "Address",
    },
    Column {
        key: "port",
        label: "Port",
    },
    Column {
        key: "subscription",
        label: "Subscription",
    },
    Column {
        key: "test",
        label: "Test result",
    },
    Column {
        key: "delay",
        label: "Real delay",
    },
    Column {
        key: "tcp",
        label: "TCP",
    },
    Column {
        key: "icmp",
        label: "ICMP",
    },
    Column {
        key: "download",
        label: "Download",
    },
    Column {
        key: "upload",
        label: "Upload",
    },
    Column {
        key: "connect",
        label: "Connect time",
    },
    Column {
        key: "ttfb",
        label: "TTFB",
    },
    Column {
        key: "http",
        label: "HTTP status",
    },
    Column {
        key: "country",
        label: "Country",
    },
    Column {
        key: "asn",
        label: "ASN",
    },
    Column {
        key: "network",
        label: "Transport",
    },
    Column {
        key: "tls",
        label: "TLS",
    },
    Column {
        key: "sni",
        label: "SNI",
    },
    Column {
        key: "tested",
        label: "Last tested",
    },
    Column {
        key: "updated",
        label: "Updated",
    },
];
pub const SUB_COLUMNS: &[Column] = &[
    Column {
        key: "source",
        label: "Source type",
    },
    Column {
        key: "created",
        label: "Created",
    },
    Column {
        key: "updated",
        label: "Updated",
    },
];

#[derive(Clone, Debug, PartialEq)]
pub struct TableState {
    pub query: String,
    pub sort: String,
    pub descending: bool,
    pub columns: BTreeSet<String>,
    pub page: usize,
    pub per_page: usize,
    pub protocol: String,
    pub state: String,
    pub show_deleted: bool,
    pub subscription: String,
    pub test: String,
    pub max_delay: String,
    pub country: String,
    pub source: String,
}
impl TableState {
    pub fn new(subscriptions: bool) -> Self {
        Self {
            query: String::new(),
            sort: "name".into(),
            descending: false,
            columns: if subscriptions {
                ["source"].as_slice()
            } else {
                ["protocol", "address", "delay"].as_slice()
            }
            .iter()
            .map(|key| (*key).into())
            .collect(),
            page: 1,
            per_page: 25,
            protocol: String::new(),
            state: String::new(),
            show_deleted: false,
            subscription: String::new(),
            test: String::new(),
            max_delay: String::new(),
            country: String::new(),
            source: String::new(),
        }
    }
    pub fn parse(query: &str, subscriptions: bool) -> Self {
        let values =
            serde_urlencoded::from_str::<BTreeMap<String, String>>(query.trim_start_matches('?'))
                .unwrap_or_default();
        let mut state = Self::new(subscriptions);
        let columns = if subscriptions {
            SUB_COLUMNS
        } else {
            CONFIG_COLUMNS
        };
        let get = |key: &str| values.get(key).cloned().unwrap_or_default();
        state.query = get("q");
        let sort = get("sort");
        if sort == "ref"
            || sort == "name"
            || sort == if subscriptions { "count" } else { "state" }
            || columns.iter().any(|col| col.key == sort)
        {
            state.sort = sort;
        }
        state.descending = get("order") == "desc";
        if let Some(value) = values.get("columns") {
            state.columns = value
                .split(',')
                .filter(|key| columns.iter().any(|col| col.key == *key))
                .map(str::to_string)
                .collect();
        }
        state.page = get("page")
            .parse::<usize>()
            .unwrap_or(1)
            .clamp(1, 1_000_000);
        if let Ok(size @ (25 | 50 | 100)) = get("per_page").parse() {
            state.per_page = size;
        }
        if subscriptions {
            state.source = get("source");
        } else {
            state.protocol = get("protocol");
            state.subscription = get("subscription");
            state.country = get("country");
            state.show_deleted = get("show_deleted") == "true";
            let status = get("state");
            if ["enabled", "disabled", "deleted"].contains(&status.as_str()) {
                state.show_deleted |= status == "deleted";
                state.state = status;
            }
            let test = get("test");
            if ["passed", "failed", "untested"].contains(&test.as_str()) {
                state.test = test;
            }
            let max = get("max_delay");
            if max.parse::<u64>().is_ok() {
                state.max_delay = max;
            }
        }
        state
    }
    pub fn query_string(&self, subscriptions: bool) -> String {
        let mut fields = BTreeMap::<&str, String>::new();
        for (key, value) in [("q", &self.query), ("sort", &self.sort)] {
            if !(value.is_empty() || key == "sort" && value == "name") {
                fields.insert(key, value.clone());
            }
        }
        if self.descending {
            fields.insert("order", "desc".into());
        }
        if self.columns != Self::new(subscriptions).columns {
            fields.insert(
                "columns",
                self.columns.iter().cloned().collect::<Vec<_>>().join(","),
            );
        }
        if self.page > 1 {
            fields.insert("page", self.page.to_string());
        }
        if self.per_page != 25 {
            fields.insert("per_page", self.per_page.to_string());
        }
        if subscriptions {
            if !self.source.is_empty() {
                fields.insert("source", self.source.clone());
            }
        } else {
            if self.show_deleted {
                fields.insert("show_deleted", "true".into());
            }
            for (key, value) in [
                ("protocol", &self.protocol),
                ("state", &self.state),
                ("subscription", &self.subscription),
                ("test", &self.test),
                ("max_delay", &self.max_delay),
                ("country", &self.country),
            ] {
                if !value.is_empty() {
                    fields.insert(key, value.clone());
                }
            }
        }
        serde_urlencoded::to_string(fields).unwrap_or_default()
    }
    pub fn sort_by(&mut self, key: &str) {
        self.descending = self.sort == key && !self.descending;
        self.sort = key.into();
        self.page = 1;
    }
    pub fn share_filter(&self) -> xrat_web::configs::sharing::ConfigFilter {
        xrat_web::configs::sharing::ConfigFilter {
            q: self.query.clone(),
            sort: self.sort.clone(),
            order: if self.descending { "desc" } else { "asc" }.into(),
            protocol: self.protocol.clone(),
            state: self.state.clone(),
            show_deleted: self.show_deleted,
            subscription: self.subscription.clone(),
            test: self.test.clone(),
            max_delay: self.max_delay.parse().ok(),
            country: self.country.clone(),
        }
    }
    pub fn filter_key(&self) -> String {
        format!(
            "{:?}",
            (
                &self.query,
                &self.protocol,
                &self.state,
                self.show_deleted,
                &self.subscription,
                &self.test,
                &self.max_delay,
                &self.country,
                &self.source
            )
        )
    }
}
fn read_query(path: &str, subscriptions: bool) -> TableState {
    let query = web_sys::window()
        .filter(|w| w.location().pathname().ok().as_deref() == Some(path))
        .and_then(|w| w.location().search().ok())
        .unwrap_or_default();
    TableState::parse(&query, subscriptions)
}
pub fn use_table_state(subscriptions: bool, path: &'static str) -> Signal<TableState> {
    let state = use_signal(|| read_query(path, subscriptions));
    let _listener =
        use_hook(move || std::rc::Rc::new(QueryListener::new(state, path, subscriptions)));
    use_effect(move || {
        let next = state().query_string(subscriptions);
        if let Some(window) = web_sys::window()
            && window.location().pathname().ok().as_deref() == Some(path)
            && let Ok(history) = window.history()
        {
            let target = if next.is_empty() {
                path.to_string()
            } else {
                format!("{path}?{next}")
            };
            let current = format!("{path}{}", window.location().search().unwrap_or_default());
            if current != target {
                let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&target));
            }
        }
    });
    state
}
struct QueryListener {
    callback: Closure<dyn FnMut(web_sys::Event)>,
}
impl QueryListener {
    fn new(mut state: Signal<TableState>, path: &'static str, subscriptions: bool) -> Self {
        let callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
            if web_sys::window()
                .is_some_and(|w| w.location().pathname().ok().as_deref() == Some(path))
            {
                state.set(read_query(path, subscriptions));
            }
        }) as Box<dyn FnMut(web_sys::Event)>);
        if let Some(window) = web_sys::window() {
            let _ = window
                .add_event_listener_with_callback("popstate", callback.as_ref().unchecked_ref());
        }
        Self { callback }
    }
}
impl Drop for QueryListener {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            let _ = window.remove_event_listener_with_callback(
                "popstate",
                self.callback.as_ref().unchecked_ref(),
            );
        }
    }
}
