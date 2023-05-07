use std::net::Ipv4Addr;
use std::process::Command;
use std::io;
use tun::Device;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::tun_device::{TunReader, TunWriter};

pub fn create_tun(
    address: Ipv4Addr,
    netmask: Ipv4Addr,
    gateway: Ipv4Addr,
) -> crate::error::Result<(TunWriter, TunReader)> {
    println!("========TUN网卡配置========");
    let mut config = tun::Configuration::default();

    config
        .destination(gateway)
        .address(address)
        .netmask(netmask)
        .mtu(1420)
        .up();

    let dev = tun::create(&config).unwrap();
    config_ip(dev.name(), address, netmask, gateway)?;

    let packet_information = dev.has_packet_information();
    let queue = dev.queue(0).unwrap();
    let reader = queue.reader();
    let writer = queue.writer();
    println!("name:{:?}", dev.name());
    println!("========TUN网卡配置========");
    Ok((
        TunWriter(writer, packet_information, Arc::new(Mutex::new(dev))),
        TunReader(reader, packet_information),
    ))
}

pub(crate) fn config_ip(name: &str, address: Ipv4Addr, netmask: Ipv4Addr, gateway: Ipv4Addr) -> io::Result<()> {
    let up_eth_str: String = format!("ifconfig {} {:?} {:?} up ", name, address, gateway);
    let route_add_str: String = format!(
        "sudo route -n add -net {:?} -netmask {:?} {:?}",
        address, netmask, gateway
    );
    let up_eth_out = Command::new("sh")
        .arg("-c")
        .arg(up_eth_str)
        .output()
        .expect("sh exec error!");
    if !up_eth_out.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("设置网络地址失败: {:?}", up_eth_out)));
    }
    let if_config_out = Command::new("sh")
        .arg("-c")
        .arg(route_add_str)
        .output()
        .expect("sh exec error!");
    if !if_config_out.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("添加路由失败: {:?}", if_config_out)));
    }
    Ok(())
}
