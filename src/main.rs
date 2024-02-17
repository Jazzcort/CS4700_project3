use clap::{Parser, ValueEnum};
use std::net::UdpSocket;
use router::{Router, GLOBAL_ROUTER, GLOBAL_TABLE};
use serde_json::Value;

use crate::{
    ipv4::{
        apply_mask, apply_mask_prefix, divide_prefix, netmask_digit, netnask_increase, to_decimal,
    },
    routing_table::{Network, Origin, Table},
};

mod ipv4;
mod router;
mod routing_table;
#[macro_use]
extern crate lazy_static;

#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Cli {
    asn: u8,
    neighbors: Vec<String>,
}

fn main() {
    // Parse command line arguments into a `Cli` struct.
    let cli = Cli::parse();
    // Iterate over each neighbor specified in the command line arguments.
    for neighbor in &cli.neighbors {
        let neighbor_information: Vec<_> = neighbor.split('-').collect();
        // Destructure the vector to individual variables for readability.
        let (neighbor_port, neighbor_ip, neighbor_relation) = (
            neighbor_information[0],
            neighbor_information[1],
            neighbor_information[2],
        );

        // Attempt to add the neighbor to the global router. `Router::add_neighbor` updates
        // the global router instance with the new neighbor's details.
        match Router::add_neighbor(neighbor_ip, neighbor_port, neighbor_relation, cli.asn) {
            Ok(()) => {
                println!("Router created successfully");
            }
            Err(e) => {
                println!("Error : {}", e);
            }
        }
    }

    // Start the router.
    Router::start_router();
}
