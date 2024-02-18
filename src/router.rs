use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::net::UdpSocket;

use crate::routing_table::{Network, Table};


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

lazy_static! {
    // Create a globally accessible `Router` instance wrapped in a `Mutex` for thread safe mutable access.
    pub static ref GLOBAL_ROUTER: Mutex<Router> = Mutex::new(Router {
        asn: 0,
        sockets: HashMap::new(),
        ports: HashMap::new(),
        relations: HashMap::new(),
    });
    // Create neighbor vector for storing all the neighbors
    pub static ref GLOBAL_PEER: Mutex<Vec<String>> = Mutex::new(vec![]);

    // Create a globally accessible `Table` instance wrapped in a `Mutex` for thread safe mutable access.
    // During the BGP process, the router will 'update' this table with new routes and use it to make routing decisions.
    pub static ref GLOBAL_TABLE: Mutex<Table> = Mutex::new(Table::new());
}

/// A message that can be sent between routers.
#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,
    dst: String,
    r#type: String,
    msg: Value,
}

impl Router {
    pub fn assign_asn(asn: u8) -> Result<(), String> {
        let mut router = GLOBAL_ROUTER
            .lock()
            .map_err(|e| format!("Failed to lock router: {}", e))?;

        router.asn = asn;
        Ok(())
    }

    // Creates a new Router instance
    pub fn add_neighbor(
        neighbor_addr: &str,
        neighbor_port: &str,
        neighbor_relation: &str
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

        router.sockets.insert(neighbor_addr.to_string(), udp_socket);
        router
            .ports
            .insert(neighbor_addr.to_string(), neighbor_port.to_string());
        router.relations.insert(neighbor_addr.to_string(), relation);
        let mut peers = GLOBAL_PEER
            .lock()
            .map_err(|e| format!("{e} -> failed to lock peer vector"))?;
        peers.push(neighbor_addr.to_string());

        Ok(())
    }

    /**
     * This function is used to start Router
     * It sends a handshake to each registed neighbor at first,
     * and then keep listening each scoket for any incoming messages
     */
    pub fn start_router() -> Result<(), String> {
        let router = GLOBAL_ROUTER
            .lock()
            .map_err(|e| format!("Failed to lock router: {}", e))?;

        let peers = GLOBAL_PEER
            .lock()
            .map_err(|e| format!("{e} -> failed to lock peer vector"))?;

        dbg!(&router);

        // Iterate through all the registered neighbors and do the handshake
        for ip_addr in peers.iter() {
            let (socket, port) = (
                router.sockets.get(ip_addr).unwrap(),
                router.ports.get(ip_addr).unwrap(),
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
            for ip_addr in peers.iter() {
                let socket = router.sockets.get(ip_addr).unwrap();
                // Listen to any incoming message
                match socket.recv(&mut buf) {
                    Ok(_) => {
                        let msg = Router::read_to_string(&mut buf)?;
                        buf.fill(0);
                        dbg!(&msg);
                        let mut json_obj: Message = serde_json::from_str(&msg)
                            .map_err(|e| format!("{e} -> failed to parse JSON object"))?;
                        match json_obj.r#type.as_str() {
                            "update" => {
                                router.handle_update_message(&mut json_obj, ip_addr)?;
                            }
                            "withdraw" => {
                                router.handle_withdraw_message(&json_obj, ip_addr)?;
                            }
                            "dump" => {
                                router.handle_dump_message(&json_obj, ip_addr)?;
                            }
                            "data" => {
                                router.handle_data_message(&json_obj, ip_addr)?;
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

    /// Reads a buffer and returns a string.
    fn read_to_string(buf: &mut [u8]) -> Result<String, String> {
        for ind in 0..buf.len() {
            if buf[ind] == 0 {
                return Ok(String::from_utf8_lossy(&buf[..ind]).to_string());
            }
        }
        Err(format!("Data incomplete"))
    }

    /// Processes and forwards "update" messages according to BGP policies.
    /// # Arguments
    /// * `json_obj` - A mutable reference to the received "update" message.
    /// * `ip_addr` - The IP address of the neighbor that sent the "update" message.
    fn handle_update_message(
        &self,
        json_obj: &mut Message,
        ip_addr: &str, // neighbor ip address
    ) -> Result<(), String> {
        let socket = self.sockets.get(ip_addr).unwrap();
        // Create new ASPath array
        let mut new_arr: Vec<Value> = vec![json!(self.asn.clone())];
        if let Value::Array(arr) = json_obj.msg["ASPath"].clone() {
            for val in arr.iter() {
                new_arr.push(val.clone());
            }
        }
        // Include peer in the message for table row update
        json_obj.msg["peer"] = json!(ip_addr);
        // Get global table for updating
        let mut table = GLOBAL_TABLE
            .lock()
            .map_err(|e| format!("{e} -> failed to lock the table"))?;
        // Update the table
        let net: Network = serde_json::from_str(&json_obj.msg.to_string()).unwrap();
        table.update(net);

        // Logic for forwarding the announcement
        // Decide who to forward the announcement to
        match self.relations[ip_addr] {
            // If sender is my customer, I will forward to everyone
            NeighborType::Cust => {
                for (nei_ip, nei_port) in self.ports.iter() {
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
                                "ASPath": json!(new_arr),
                            }

                        });
                        socket.send_to(update_msg.to_string().as_bytes(),format!("127.0.0.1:{nei_port}")).map_err(|e| format!("{e} -> failed to send update message to {ip_addr} with 127.0.0.1:{nei_port}"))?;
                    }
                }
            }
            // If sender is not my customer, I will only forward your announcement to my customer
            _ => {
                for (nei_ip, nei_port) in self.ports.iter() {
                    // Send the "update" message to every neighbor except the origin
                    if nei_ip != ip_addr {
                        // Forward announcement only to my customer
                        match self.relations[nei_ip] {
                            NeighborType::Cust => {
                                // Customize update message
                                let update_msg = json!({
                                    "src": format!("{}{}", &nei_ip[..nei_ip.len() - 1], "1"),
                                    "dst": nei_ip,
                                    "type": "update",
                                    "msg": {
                                        "network": &json_obj.msg["network"],
                                        "netmask": &json_obj.msg["netmask"],
                                        "ASPath": json!(new_arr),
                                    }

                                });
                                socket.send_to(update_msg.to_string().as_bytes(), format!("127.0.0.1:{nei_port}")).map_err(|e| format!("{e} -> failed to send update message to {ip_addr} with 127.0.0.1:{nei_port}"))?;
                            }
                            // Do nothing, if the neighbor is not my customer
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Processes and forwards "withdraw" messages according to BGP policies.
    /// # Arguments
    /// * `json_obj` - A reference to the received "withdraw" message.
    /// * `ip_addr` - The IP address of the neighbor that sent the "withdraw" message.
    fn handle_withdraw_message(
        &self,
        json_obj: &Message,
        ip_addr: &str,
    ) -> Result<(), String> {
        for _network in json_obj.msg.as_array().unwrap() {
            let network = _network["network"].as_str().unwrap();
            let netmask = _network["netmask"].as_str().unwrap();
            
        // Get the socket for the neighbor
        let socket = self.sockets.get(ip_addr).unwrap();
        // Get global table for updating
        let mut table = GLOBAL_TABLE
            .lock()
            .map_err(|e| format!("{e} -> failed to lock the table"))?;


        // Update the table
        table.withdraw(
            network,
            netmask,
            ip_addr,
        );

        // Logic for forwarding the announcement
        // Decide who to forward the announcement to
        match self.relations[ip_addr] {
            // If sender is my customer, I will forward to everyone
            NeighborType::Cust => {
                for (nei_ip, nei_port) in self.ports.iter() {
                    // Send the "withdraw" message to every neighbor except the origin
                    if nei_ip != ip_addr {
                        // Customize withdraw message
                        let withdraw_msg = json!({
                            "src": format!("{}{}", &nei_ip[..nei_ip.len() - 1], "1"),
                            "dst": nei_ip,
                            "type": "withdraw",
                            "msg": [{
                                "network": _network["network"],
                                "netmask": _network["netmask"],
                            }]

                        });
                        socket.send_to(withdraw_msg.to_string().as_bytes(),format!("127.0.0.1:{nei_port}")).map_err(|e| format!("{e} -> failed to send update message to {ip_addr} with 127.0.0.1:{nei_port}"))?;
                    }
                }
            }
            // If sender is not my customer, I will only forward your announcement to my customer
            _ => {
                for (nei_ip, nei_port) in self.ports.iter() {
                    // Send the "withdraw" message to every neighbor except the origin
                    if nei_ip != ip_addr {
                        // Forward announcement only to my customer
                        match self.relations[nei_ip] {
                            NeighborType::Cust => {
                                // Customize withdraw message
                                let withdraw_msg = json!({
                                    "src": format!("{}{}", &nei_ip[..nei_ip.len() - 1], "1"),
                                    "dst": nei_ip,
                                    "type": "withdraw",
                                    "msg": [{
                                        "network": _network["network"],
                                        "netmask": _network["netmask"],
                                    }]

                                });
                                socket.send_to(withdraw_msg.to_string().as_bytes(), format!("127.0.0.1:{nei_port}")).map_err(|e| format!("{e} -> failed to send update message to {ip_addr} with 127.0.0.1:{nei_port}"))?;
                            }
                            // Do nothing, if the neighbor is not my customer
                            _ => {}
                        }
                    }
                }
            }
        }
    }

        Ok(())
    }

    /// Processes and forwards "data" messages according to BGP policies.
    /// # Arguments
    /// * `json_obj` - A reference to the received "data" message.
    /// * `ip_addr` - The IP address of the neighbor that sent the "data" message.
    fn handle_data_message(
        &self,
        json_obj: &Message,
        ip_addr: &str
    ) -> Result<(), String> {
        let src_port = self.ports[ip_addr].clone();
        let socket = self.sockets.get(ip_addr).unwrap();
        // We check if we can find the best route in the table
        match Table::best_route(&json_obj.dst) {
            Ok(peer_ip) => {
                let data_message = json!({
                    "src": json_obj.src,
                    "dst": json_obj.dst,
                    "type": "data",
                    "msg": json_obj.msg
                });

                // Port that we will send the message to
                let peer_port = self.ports.get(&peer_ip).unwrap();

                match self.relations[ip_addr] {
                    // If source is my customer, I will forward to everyone
                    NeighborType::Cust => {
                        socket
                            .send_to(
                                data_message.to_string().as_bytes(),
                                format!("127.0.0.1:{}", peer_port),
                            )
                            .map_err(|e| format!("{e} -> failed to send the data message"))?;
                    }
                    _ => {
                        // If source is not my customer, I will only forward your announcement to my customer
                        match self.relations.get(&peer_ip).unwrap() {
                            NeighborType::Cust => {
                                socket
                                    .send_to(
                                        data_message.to_string().as_bytes(),
                                        format!("127.0.0.1:{}", peer_port),
                                    )
                                    .map_err(|e| {
                                        format!("{e} -> failed to send the data message")
                                    })?;
                            }
                            _ => {
                                let no_route_message = json!({
                                    "src": format!("{}{}", &ip_addr[..ip_addr.len() - 1], "1"),
                                    "dst": json_obj.src,
                                    "type": "no route",
                                    "msg": {}
                                });

                                socket
                                    .send_to(
                                        no_route_message.to_string().as_bytes(),
                                        format!("127.0.0.1:{}", src_port),
                                    )
                                    .map_err(|e| {
                                        format!("{e} -> failed to send the no route message")
                                    })?;
                            }
                        }
                    }
                }
            }
            Err(_) => {
                let no_route_message = json!({
                    "src": format!("{}{}", &ip_addr[..ip_addr.len() - 1], "1"),
                    "dst": json_obj.src,
                    "type": "no route",
                    "msg": {}
                });

                socket
                    .send_to(
                        no_route_message.to_string().as_bytes(),
                        format!("127.0.0.1:{}", src_port),
                    )
                    .map_err(|e| format!("{e} -> failed to send the no route message"))?;
            }
        }
        Ok(())
    }

    /// Handles a "dump" message received from a neighbor and responds with a "table" message.
    /// This "table" message contains a copy of the current routing table.
    /// # Arguments
    /// * `message` - A reference to the received "dump" message.
    /// * `ip_addr` - neighbor's ip address
    /// # Returns
    /// * `Result<(), String>` - Ok(()) if the response was successfully sent, or Err(String) with an error message if not.
    fn handle_dump_message(
        &self,
        message: &Message,
        ip_addr: &str,
    ) -> Result<(), String> {
        let socket = self.sockets.get(ip_addr).unwrap();
        let src_port = self.ports[ip_addr].clone();
        let table = GLOBAL_TABLE
            .lock()
            .map_err(|e| format!("{e} -> failed to lock the table"))?;
        // Generate response to send back to the sender
        let response = json!({
            "src": message.dst,
            "dst": message.src,
            "type": "table",
            "msg": json!(table.get_table().clone()) // Copy rounting table from global router
        });

        // Find the correct port to send it back
        socket
            .send_to(
                response.to_string().as_bytes(),
                format!("127.0.0.1:{src_port}"),
            )
            .map_err(|e| format!("Failed to send table message: {}", e))?;

        Ok(())
    }
}
