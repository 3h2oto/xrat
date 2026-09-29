//! Bridges runtime tuning and routing app-config sections to generated engine
//! options, plus inbound `listen_interface` resolution.

pub(crate) use std::net::IpAddr;
pub(crate) use std::path::Path;
pub(crate) use std::process::Command;

pub(crate) use crate::app::AppError;
pub(crate) use crate::app::config::{
    DnsHostValue, DnsSettings, RouteList, RoutingSettings, RuntimeSettings,
};
pub(crate) use serde_json::{Value, json};
pub(crate) use url::Url;
pub(crate) use xrat_engines::singbox::{SingboxDnsConfig, SingboxRouteList, SingboxRoutingOptions};
pub(crate) use xrat_engines::xray::{
    FragmentOptions, MuxOptions, XrayCompatibilityPolicy, XrayCompatibilityTarget, XrayDnsConfig,
    XrayDnsHostValue, XrayGenOptions, XrayRouteList, XrayRoutingOptions,
};

pub(crate) const SINGBOX_LOCAL_DNS_TAG: &str = "xrat-dns-local";
pub(crate) const SINGBOX_HOSTS_DNS_TAG: &str = "xrat-dns-hosts";
