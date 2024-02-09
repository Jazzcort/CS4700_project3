use std::{any::Any, net::UdpSocket};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Number, Value};
use std::sync::Mutex;

use crate::routing_table::Table;


// enum Type {
//     Update,
//     Withdraw,
//     Data,

// }


/// Represents the type of relationship with a neighbor.
#[derive(Debug)]
pub enum NeighborType {
    /// Indicates a peer relationship.
    Peer,
    /// Indicates a customer relationship.
    Cust,
    /// Indicates a provider relationship.
    Prov,
}

/// A router that maintains connections, port mappings, and relationships with neighbors.
#[derive(Debug)]
pub struct Router {
    asn: u8,

    /// Maps neighbor IP addresses to their respective UDP sockets for communication.
    /// This allows the router to send and receive packets to and from each neighbor.
    sockets: HashMap<String, UdpSocket>,

    /// Maps neighbor IP addresses to the ports they are associated with.
    /// This is useful for managing network topology and understanding
    /// where messages should be sent or received.
    ports: HashMap<String, String>,

    /// Maps neighbor IP addresses to their relationship type with this router.
    /// The relationship can be one of Peer, Customer (Cust), or Provider (Prov),
    /// and affects routing decisions and policy.
    relations: HashMap<String, NeighborType>,
}

// Create a globally accessible `Router` instance wrapped in a `Mutex` for thread safe mutable access.
lazy_static! {
    pub static ref GLOBAL_ROUTER: Mutex<Router> = Mutex::new(Router {
        asn: 0,
        sockets: HashMap::new(),
        ports: HashMap::new(),
        relations: HashMap::new(),
    });
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,
    dst: String,
    r#type: String,
    msg: Value
}


impl Router {
    // Creates a new Router instance
    pub fn add_neighbor(neighbor_addr: &str, neighbor_port: &str, neighbor_relation: &str, asn: u8) -> Result<(), String> {
        // Lock the mutex here
        let mut router = GLOBAL_ROUTER.lock().map_err(|e| format!("Failed to lock router: {}", e))?;

        // Get the relationship
        let relation = match neighbor_relation {
            "cust" => NeighborType::Cust,
            "peer" => NeighborType::Peer,
            "prov" => NeighborType::Prov,
            _ => return Err("Invalid neighbor relationship".to_string()),
        };

        // Set up the UDP socket at local for the neighbor router
        let udp_socket = UdpSocket::bind("127.0.0.1:0").map_err(|e| format!("{e} -> failed to create neighbor socket"))?;
        
        router.asn = asn;
        router.sockets.insert(neighbor_addr.to_string(), udp_socket);
        router.ports.insert(neighbor_addr.to_string(), neighbor_port.to_string());
        router.relations.insert(neighbor_addr.to_string(), relation);

        Ok(())
    }

    // fn read_to_string(buf: &mut [u8]) -> Result<String, String> {
    //     for ind in 0..buf.len() {
    //         if buf[ind] == 0 {
    //             return Ok(String::from_utf8_lossy(&buf[..ind]).to_string())
    //         }
    //     }
    //     return Err(format!("Data incomplete"));
    // }

    // pub fn register_neighbor(&mut self, ip: &str, port: &str, neighbor_type: NeighborType) {
    //     self.sender.insert(ip.to_string(), (neighbor_type, format!("127.0.0.1:{port}")));
    // }

    // pub fn keep_listen(&mut self) {
    //     let mut buf: [u8; 2048] = [0; 2048];
        
    //     loop {

    //         self.receiver.recv(&mut buf).unwrap();
    //         let res = Router::read_to_string(&mut buf).unwrap();

    //         println!("{}", &res);
    //         // let a = serde_json::from_slice(res.as_bytes());
    //         dbg!(res.len());
    //         let mut v: Message = serde_json::from_str(&res).expect("failed!!!");

    //         // println!("{}", v.msg["ASPath"]);
    //         if let Value::Array(mut a) = v.msg["ASPath"].clone() {
    //             a.push(json!(2));
    //             println!("{:?}", a);
    //             v.msg["ASPath"] = json!(a);
    //         }
    //         match v.r#type.as_str() {
    //              "update"=> {
    //                 let src = v.src.clone();
    //                 dbg!(&src);
    //                 let (tmp, addr) = self.sender.get_mut(&src).unwrap();
    //                 self.receiver.send_to(json!(v).to_string().as_bytes(), "127.0.0.1:7878".to_string()).unwrap();
    //              },

    //             _ => {}

    //         }

    //         println!("{:?}", v);

    //         // let j = json!({"a": "123"});
    //     }

    // }
}