use std::net::UdpSocket;
use clap::{Parser, ValueEnum};

use router::Router;

use crate::{ipv4::{apply_mask, apply_mask_prefix, netmask_digit, to_decimal}, routing_table::{Network, Origin, Table}};

mod ipv4;
mod routing_table;
mod router;
#[macro_use]
extern crate lazy_static;

#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Cli {
    asn: u8
}

fn main() {
    let cli = Cli::parse();
    dbg!(cli);
    // dbg!(ipv4::apply_mask("128.42.222.198", "255.255.128.0"));
    // dbg!(ipv4::apply_mask("128.42.128.0", "255.255.0.0"));
    // dbg!(ipv4::to_decimal("128.42.128.0"));

    // dbg!(ipv4::check_match(
    //     "128.42.128.0",
    //     "255.255.128.0",
    //     "128.42.222.198"
    // ));

    // dbg!(ipv4::netmask_digit("255.255.128.0"));
    // let net1: Network = Network::new(
    //     format!("192.168.0.0"),
    //     format!("192.168.2.0"),
    //     format!("255.255.255.0"),
    //     100,
    //     true,
    //     vec![1],
    //     Origin::Egp,
    // );
    // let net2: Network = Network::new(
    //     format!("192.168.0.0"),
    //     format!("192.168.3.0"),
    //     format!("255.255.255.0"),
    //     100,
    //     true,
    //     vec![1],
    //     Origin::Egp,
    // );
    // let net3: Network = Network::new(
    //     format!("192.168.0.0"),
    //     format!("192.168.0.0"),
    //     format!("255.255.254.0"),
    //     100,
    //     true,
    //     vec![1],
    //     Origin::Egp,
    // );
    // let net4: Network = Network::new(
    //     format!("192.168.0.0"),
    //     format!("192.168.4.0"),
    //     format!("255.255.252.0"),
    //     100,
    //     true,
    //     vec![1],
    //     Origin::Egp,
    // );
    // let mut t = Table::new();
    // t.update(net1.clone());
    // t.update(net2);
    // t.update(net1);
    // t.update(net3);
    // // t.update(net4);
    // dbg!(&t);
    // // dbg!(Table::is_aggregable(&net1, &net2));
    // // dbg!(apply_mask_prefix(&net1.net_prefix, &net1.netmask));
    // // dbg!(apply_mask_prefix(&net2.net_prefix, &net2.netmask));
    // dbg!(ipv4::to_decimal("255.255.128.0"));
    // dbg!(ipv4::to_ipv4(ipv4::to_decimal("255.255.168.1")));
    // // dbg!(ipv4::apply_mask("255.255.128.0", "255"))
    // dbg!(ipv4::to_decimal("255.255.192.0") << 1);
    // dbg!(ipv4::to_ipv4(ipv4::to_decimal("255.255.192.0") << 1));

    // dbg!(format!("{:032b}", apply_mask("173.98.112.0", "255.255.248.0")));
    // dbg!(netmask_digit("255.255.224.0"));
    // dbg!(format!("{:08b}", 112));
    // dbg!(format!("{:032b}", to_decimal("255.255.253.0")));


    let mut router = Router::new("127.0.0.1:5005".to_string()).unwrap();
    router.register_neighbor("192.168.0.2", "63456", router::NeighborType::Cust);
    router.register_neighbor("172.168.0.2", "63886", router::NeighborType::Cust);
    dbg!(&router.sender);
    router.keep_listen();

    // let a = UdpSocket::bind("127.0.0.1:7777").unwrap();
    // let b = UdpSocket::bind("127.0.0.1:8888").unwrap();
}
