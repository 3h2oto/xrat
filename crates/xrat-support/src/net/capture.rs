use std::net::IpAddr;

#[derive(Debug, Clone, Default)]
pub struct NetworkSnapshot {
    pub adapters: Vec<NetworkAdapter>,
    pub routes: Vec<NetworkRoute>,
}
#[derive(Debug, Clone)]
pub struct NetworkAdapter {
    pub name: String,
    pub identity: String,
    pub luid: u64,
    pub index: u32,
    pub physical: bool,
    pub addresses: Vec<String>,
    pub dns: Vec<IpAddr>,
}
#[derive(Debug, Clone)]
pub struct NetworkRoute {
    pub luid: u64,
    pub address: IpAddr,
    pub prefix: u8,
}
