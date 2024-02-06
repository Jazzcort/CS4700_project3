use crate::{ipv4::apply_mask_prefix, routing_table::{Network, Origin, Table}};

mod ipv4;
mod routing_table;
#[macro_use]
extern crate lazy_static;

fn main() {
    dbg!(ipv4::apply_mask("128.42.222.198", "255.255.128.0"));
    dbg!(ipv4::apply_mask("128.42.128.0", "255.255.128.0"));
    dbg!(ipv4::to_decimal("128.42.128.0"));

    dbg!(ipv4::check_match(
        "128.42.128.0",
        "255.255.128.0",
        "128.42.222.198"
    ));

    dbg!(ipv4::netmask_digit("255.255.128.0"));
    let net1: Network = Network::new(
        format!("192.168.0.0"),
        format!("192.168.2.0"),
        format!("255.255.255.0"),
        100,
        true,
        vec![1],
        Origin::Egp,
    );
    let net2: Network = Network::new(
        format!("192.168.0.0"),
        format!("192.168.3.0"),
        format!("255.255.255.0"),
        100,
        true,
        vec![1],
        Origin::Egp,
    );

    // dbg!(Table::is_aggregable(&net1, &net2));
    // dbg!(apply_mask_prefix(&net1.net_prefix, &net1.netmask));
    // dbg!(apply_mask_prefix(&net2.net_prefix, &net2.netmask));
    dbg!(ipv4::to_decimal("255.255.128.0"));
    dbg!(ipv4::to_ipv4(ipv4::to_decimal("255.255.168.1")));
    // dbg!(ipv4::apply_mask("255.255.128.0", "255"))
    dbg!(ipv4::to_decimal("255.255.192.0") << 1);
    dbg!(ipv4::to_ipv4(ipv4::to_decimal("255.255.192.0") << 1));
}
