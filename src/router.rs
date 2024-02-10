use serde::{Deserialize, Serialize};
use serde_json::{json, Number, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::{any::Any, net::UdpSocket};

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
    /// The routing table for this router.
    /// This table contains information about the network topology and is used
    /// to make routing decisions.
    routing_table: Table,                       
}

// Create a globally accessible `Router` instance wrapped in a `Mutex` for thread safe mutable access.
lazy_static! {
    pub static ref GLOBAL_ROUTER: Mutex<Router> = Mutex::new(Router {
        asn: 0,
        sockets: HashMap::new(),
        ports: HashMap::new(),
        relations: HashMap::new(),
        routing_table: Table::new(),
    });
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,
    dst: String,
    r#type: String,
    msg: Value,
}


impl Router {
    // Creates a new Router instance
    pub fn add_neighbor(
        neighbor_addr: &str,
        neighbor_port: &str,
        neighbor_relation: &str,
        asn: u8,
    ) -> Result<(), String> {
        // Lock the mutex here
        let mut router = GLOBAL_ROUTER
            .lock()
            .map_err(|e| format!("Failed to lock router: {}", e))?;

        // Get the relationship
        let relation = match neighbor_relation {
            "cust" => NeighborType::Cust,
            "peer" => NeighborType::Peer,
            "prov" => NeighborType::Prov,
            _ => return Err("Invalid neighbor relationship".to_string()),
        };

        // Set up the UDP socket at local for the neighbor router
        let udp_socket = UdpSocket::bind("127.0.0.1:0")
            .map_err(|e| format!("{e} -> failed to create neighbor socket"))?;
        udp_socket
            .set_nonblocking(true)
            .map_err(|e| format!("{e} -> failed to switch to non-blockinf mode"))?;

        router.asn = asn;
        router.sockets.insert(neighbor_addr.to_string(), udp_socket);
        router.ports.insert(neighbor_addr.to_string(), neighbor_port.to_string());
        router.relations.insert(neighbor_addr.to_string(), relation);

        Ok(())
    }

    /**
     * This function is used to start Router
     * It sends a handshake to each registed neighbor at first,
     * and then keep listening each scoket for any incoming messages
     */
    pub fn start_router() -> Result<(), String> {
        let mut router = GLOBAL_ROUTER
            .lock()
            .map_err(|e| format!("Failed to lock router: {}", e))?;

        // Iterate through all the registered neighbors and do the handshake
        for ip_addr in router.relations.keys() {
            let (socket, port, relation) = (
                router.sockets.get(ip_addr).unwrap(),
                router.ports.get(ip_addr).unwrap(),
                router.relations.get(ip_addr).unwrap(),
            );
            // Our ip address for this specific port
            let local_ip = format!("{}{}", &ip_addr[..ip_addr.len() - 1], "1");
            // Create the handshake message
            let handshake_msg = json!({"src":local_ip,"dst":ip_addr,"type": "handshake","msg":{}});

            socket.send_to(handshake_msg.to_string().as_bytes(), format!("127.0.0.1:{port}")).map_err(|e| format!("{e} -> failed to send handshake message to {ip_addr} with 127.0.0.1:{port}"))?;
        }

        // Create read buffer
        let mut buf: [u8; 2048] = [0; 2048];
        loop {
            // Iterate through all the neighbors
            for ip_addr in router.relations.keys() {
                let (socket, port, relation) = (
                    router.sockets.get(ip_addr).unwrap(),
                    router.ports.get(ip_addr).unwrap(),
                    router.relations.get(ip_addr).unwrap(),
                );
               
               // Listen to any incoming message
                match socket.recv(&mut buf) {
                    Ok(_) => {
                        let msg = Router::read_to_string(&mut buf)?;
                        let json_obj: Message = serde_json::from_str(&msg)
                            .map_err(|e| format!("{e} -> failed to parse JSON object"))?;
                        match json_obj.r#type.as_str() {
                            // If the message received is type "update"
                            "update" => {
                                // Create new ASPath array
                                let mut new_arr: Vec<Value> = vec![json!(router.asn.clone())];
                                if let Value::Array(arr) = json_obj.msg["ASPath"].clone() {
                                    for val in arr.iter() {
                                        new_arr.push(val.clone());
                                    }
                                }

                                for (nei_ip, nei_port) in router.ports.iter() {
                                    // Send the "update" message to every neighbor except the origin
                                    if nei_ip != ip_addr {
                                        // Customize update message
                                        let update_msg = json!({
                                            "src": format!("{}{}", &nei_ip[..nei_ip.len() - 1], "1"),
                                            "dst": nei_ip,
                                            "type": "update",
                                            "msg": {
                                                "network": &json_obj.msg["network"],
                                                "netmask": &json_obj.msg["netmask"],
                                                // "localpref": &json_obj.msg["localpref"],
                                                "ASPath": json!(new_arr),
                                                // "origin": &json_obj.msg["origin"],
                                                // "selfOrigin": &json_obj.msg["selfOrigin"]
                                            }
                                            
                                        });
                                        socket.send_to(update_msg.to_string().as_bytes(), format!("127.0.0.1:{nei_port}")).map_err(|e| format!("{e} -> failed to send update message to {ip_addr} with 127.0.0.1:{port}"))?;
                                    }
                                    
                                }
                                
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        }
    }

    fn read_to_string(buf: &mut [u8]) -> Result<String, String> {
        for ind in 0..buf.len() {
            if buf[ind] == 0 {
                return Ok(String::from_utf8_lossy(&buf[..ind]).to_string());
            }
        }
        return Err(format!("Data incomplete"));
    }

    // pub fn handle_dump_message(message: &Message) {
        
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
