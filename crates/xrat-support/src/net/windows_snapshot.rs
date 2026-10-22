use super::{NetworkAdapter, NetworkRoute, NetworkSnapshot, windows_links};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};
use windows_sys::Win32::{
    Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_NO_DATA},
    NetworkManagement::IpHelper::*,
    Networking::WinSock::*,
};

fn ip(address: &SOCKADDR_INET) -> io::Result<IpAddr> {
    unsafe {
        match address.si_family {
            AF_INET => Ok(IpAddr::V4(Ipv4Addr::from(
                address.Ipv4.sin_addr.S_un.S_addr.to_ne_bytes(),
            ))),
            AF_INET6 => Ok(IpAddr::V6(Ipv6Addr::from(address.Ipv6.sin6_addr.u.Byte))),
            _ => Err(io::Error::other(
                "unexpected address family in Windows network snapshot",
            )),
        }
    }
}
fn socket_ip(address: &SOCKET_ADDRESS) -> io::Result<IpAddr> {
    if address.lpSockaddr.is_null() || address.iSockaddrLength < size_of::<SOCKADDR>() as i32 {
        return Err(io::Error::other("truncated Windows socket address"));
    }
    let family = unsafe { (*address.lpSockaddr).sa_family };
    let mut value = SOCKADDR_INET::default();
    unsafe {
        match family {
            AF_INET if address.iSockaddrLength >= size_of::<SOCKADDR_IN>() as i32 => {
                value.Ipv4 = std::ptr::read_unaligned(address.lpSockaddr.cast())
            }
            AF_INET6 if address.iSockaddrLength >= size_of::<SOCKADDR_IN6>() as i32 => {
                value.Ipv6 = std::ptr::read_unaligned(address.lpSockaddr.cast())
            }
            _ => return Err(io::Error::other("invalid Windows socket address")),
        }
    }
    ip(&value)
}
fn routes() -> io::Result<Vec<NetworkRoute>> {
    let mut raw = std::ptr::null_mut();
    let status = unsafe { GetIpForwardTable2(AF_UNSPEC, &mut raw) };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    if raw.is_null() {
        return Err(io::Error::other("null Windows route table"));
    }
    struct Table(*mut MIB_IPFORWARD_TABLE2);
    impl Drop for Table {
        fn drop(&mut self) {
            unsafe { FreeMibTable(self.0.cast()) };
        }
    }
    let table = Table(raw);
    let count = unsafe { (*table.0).NumEntries } as usize;
    if count > 65536 {
        return Err(io::Error::other(
            "Windows route table exceeds 65536 entries",
        ));
    }
    let rows = unsafe {
        std::slice::from_raw_parts(
            std::ptr::addr_of!((*table.0).Table).cast::<MIB_IPFORWARD_ROW2>(),
            count,
        )
    };
    rows.iter()
        .map(|row| {
            Ok(NetworkRoute {
                luid: unsafe { row.InterfaceLuid.Value },
                address: ip(&row.DestinationPrefix.Prefix)?,
                prefix: row.DestinationPrefix.PrefixLength,
            })
        })
        .collect()
}
pub fn snapshot() -> io::Result<NetworkSnapshot> {
    let mut size = 32768u32;
    let mut storage;
    for attempt in 0..3 {
        if size > 4 * 1024 * 1024 {
            return Err(io::Error::other("Windows adapter snapshot exceeds 4 MiB"));
        }
        storage = vec![0u64; (size as usize).div_ceil(8)];
        let status = unsafe {
            GetAdaptersAddresses(
                u32::from(AF_UNSPEC),
                GAA_FLAG_INCLUDE_ALL_INTERFACES,
                std::ptr::null(),
                storage.as_mut_ptr().cast(),
                &mut size,
            )
        };
        if status == ERROR_BUFFER_OVERFLOW && attempt < 2 {
            continue;
        }
        if status == ERROR_NO_DATA {
            return Ok(NetworkSnapshot {
                adapters: vec![],
                routes: routes()?,
            });
        }
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        let mut adapters = Vec::new();
        let mut next = storage.as_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();
        while !next.is_null() {
            if adapters.len() >= 4096 {
                return Err(io::Error::other("too many Windows adapters"));
            }
            let adapter = unsafe { &*next };
            let mut row = MIB_IF_ROW2 {
                InterfaceLuid: adapter.Luid,
                ..Default::default()
            };
            let status = unsafe { GetIfEntry2(&mut row) };
            if status != 0 {
                return Err(io::Error::from_raw_os_error(status as i32));
            }
            let end = row
                .Alias
                .iter()
                .position(|value| *value == 0)
                .unwrap_or(row.Alias.len());
            let name = String::from_utf16(&row.Alias[..end]).map_err(io::Error::other)?;
            let mut addresses = Vec::new();
            let mut current = adapter.FirstUnicastAddress;
            let mut count = 0;
            while !current.is_null() {
                count += 1;
                if count > 4096 {
                    return Err(io::Error::other("too many interface addresses"));
                }
                let entry = unsafe { &*current };
                if entry.DadState == IpDadStatePreferred {
                    addresses.push(format!(
                        "{}/{}",
                        socket_ip(&entry.Address)?,
                        entry.OnLinkPrefixLength
                    ));
                }
                current = entry.Next;
            }
            let mut dns = Vec::new();
            let mut current = adapter.FirstDnsServerAddress;
            while !current.is_null() {
                if dns.len() >= 256 {
                    return Err(io::Error::other("too many interface DNS servers"));
                }
                let entry = unsafe { &*current };
                dns.push(socket_ip(&entry.Address)?);
                current = entry.Next;
            }
            let luid = unsafe { adapter.Luid.Value };
            adapters.push(NetworkAdapter {
                name,
                luid,
                index: row.InterfaceIndex,
                identity: format!("{}@{luid:016x}", windows_links::guid(&row)),
                physical: row.InterfaceAndOperStatusFlags._bitfield & 1 != 0,
                addresses,
                dns,
            });
            next = adapter.Next;
        }
        return Ok(NetworkSnapshot {
            adapters,
            routes: routes()?,
        });
    }
    Err(io::Error::other("Windows adapter list changed repeatedly"))
}
