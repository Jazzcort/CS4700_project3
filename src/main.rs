use clap::Parser;
use router::Router;

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
    // Assign AS number
    match Router::assign_asn(cli.asn) {
        Ok(_) => {
            println!("Successfully assigned AS number")
        }
        Err(e) => {
            println!("{}", format!("{} -> Failed to assign AS number", e))
        }
    }
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
        match Router::add_neighbor(neighbor_ip, neighbor_port, neighbor_relation) {
            Ok(()) => {
                println!("Router created successfully");
            }
            Err(e) => {
                println!("Error : {}", e);
            }
        }
    }

    // Start the router.
    Router::start_router().unwrap();
}
