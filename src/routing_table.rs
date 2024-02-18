/// This module contains the implementation of the routing table
/// and the network struct.
use crate::{
    ipv4::{
        apply_mask, apply_mask_prefix, check_match, divide_prefix, netmask_digit, netnask_increase,
        to_decimal, to_ipv4,
    },
    router::GLOBAL_TABLE,
};
use serde::{Deserialize, Serialize};

/// This enum represents the origin of the network
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Serialize, Deserialize)]
pub enum Origin {
    IGP = 3,
    EGP = 2,
    UNK = 1,
}

/// This struct represents the network
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Network {
    peer: String,
    network: String,
    netmask: String,
    localpref: i32,
    selfOrigin: bool,
    ASPath: Vec<i32>,
    origin: Origin,
}

#[allow(non_snake_case)]
impl Network {
    // This function is used to create a new Network object.
    pub fn new(
        peer: String,
        network: String,
        netmask: String,
        localpref: i32,
        selfOrigin: bool,
        ASPath: Vec<i32>,
        origin: Origin,
    ) -> Self {
        Network {
            peer,
            network,
            netmask,
            localpref,
            selfOrigin,
            ASPath,
            origin,
        }
    }
}

/// This struct represents the routing table
#[derive(Debug, Clone, Serialize)]
pub struct Table {
    table: Vec<Network>,
}

impl Table {
    pub fn new() -> Self {
        Table { table: vec![] }
    }

    // This function updates the routing table with the new network.
    pub fn update(&mut self, mut new_net: Network) {
        // Remove the network from the table if it has the same network prefix, subnet mask, and peer IP as the given network.
        self.withdraw(&new_net.network, &new_net.netmask, &new_net.peer);
        loop {
            // Whenever we want to add the new row into table,
            // we aggregate as much as possible
            match self.aggregate(new_net.clone()) {
                Some(n) => new_net = n,
                None => break,
            }
        }
        // Add aggregated row into the table
        self.table.push(new_net)
    }

    /**
     * This function withdraw the given network from the routing table
     */
    pub fn withdraw(&mut self, network: &str, netmask: &str, peer: &str) {
        while self.disaggregate(network, netmask, peer) {}
    }

    /**
     * This function apply disaggregate mechanism to the routing table
     * return true if successfully disaggregate something,
     * false if nothing gets disaggregated.
     */
    pub fn disaggregate(&mut self, network: &str, netmask: &str, peer: &str) -> bool {
        match (0..self.table.len()).into_iter().find(|ind| {
            // Check peer IP
            self.table[*ind].peer == peer
                // Check if the given network matched this row
                && check_match(
                    &self.table[*ind].network,
                    &self.table[*ind].netmask,
                    network,
                )
        }) {
            Some(ind) => {
                // Remove the matched row from table
                let mut net = self.table.remove(ind);
                // Divide the network if the subnet mask is different
                while netmask_digit(&net.netmask) < netmask_digit(netmask) {
                    let new_netmask = netnask_increase(&net.netmask);
                    let (divided_net1, divided_net2) = divide_prefix(&net.network, &new_netmask);

                    if check_match(&divided_net1, &new_netmask, network) {
                        // Push the unmatched part back to the routing table
                        self.update(Network::new(
                            peer.to_string(),
                            divided_net2,
                            new_netmask.clone(),
                            net.localpref.clone(),
                            net.selfOrigin.clone(),
                            net.ASPath.clone(),
                            net.origin.clone(),
                        ));

                        net = Network::new(
                            peer.to_string(),
                            divided_net1,
                            new_netmask,
                            net.localpref.clone(),
                            net.selfOrigin.clone(),
                            net.ASPath.clone(),
                            net.origin.clone(),
                        );
                    } else {
                        // Push the unmatched part back to the routing table
                        self.update(Network::new(
                            peer.to_string(),
                            divided_net1,
                            new_netmask.clone(),
                            net.localpref.clone(),
                            net.selfOrigin.clone(),
                            net.ASPath.clone(),
                            net.origin.clone(),
                        ));

                        net = Network::new(
                            peer.to_string(),
                            divided_net2,
                            new_netmask,
                            net.localpref.clone(),
                            net.selfOrigin.clone(),
                            net.ASPath.clone(),
                            net.origin.clone(),
                        );
                    }
                }
                true
            }
            None => false,
        }
    }

    // This is the getter function for table
    pub fn get_table(&self) -> &Vec<Network> {
        &self.table
    }

    /**
     * This function returns the best route to the given destination
     */
    pub fn best_route(dst: &str) -> Result<String, String> {
        let table = GLOBAL_TABLE
            .lock()
            .map_err(|e| format!("{e} -> failed to lock the table (best_route)"))?;
        // Create a default Network
        let mut candidate = Network::new(
            "0".to_string(),
            "255.255.255.255".to_string(),
            "0.0.0.0".to_string(),
            0,
            false,
            table.table[0].ASPath.clone(),
            Origin::UNK,
        );
        let mut longest_prefix = 0;

        for net in table.table.iter() {
            // let (network, netmask) = (net.network, net.netmask);
            if check_match(&net.network, &net.netmask, dst) {
                let prefix_length = netmask_digit(&net.netmask);
                // Check if the subnet mask is longer than the current longest prefix
                if prefix_length > longest_prefix {
                    candidate = net.clone();
                    longest_prefix = prefix_length;
                } else if prefix_length == longest_prefix {
                    // Check localpref
                    if candidate.localpref > net.localpref {
                        continue;
                    } else if candidate.localpref < net.localpref {
                        candidate = net.clone();
                        continue;
                    }

                    // Check selfOrigin
                    if candidate.selfOrigin != net.selfOrigin {
                        if candidate.selfOrigin {
                            continue;
                        } else {
                            candidate = net.clone();
                            continue;
                        }
                    }

                    // Check ASPath
                    if candidate.ASPath.len() > net.ASPath.len() {
                        candidate = net.clone();
                        continue;
                    } else if candidate.ASPath.len() < net.ASPath.len() {
                        continue;
                    }

                    // Check origin
                    if candidate.origin > net.origin {
                        continue;
                    } else if candidate.origin < net.origin {
                        candidate = net.clone();
                        continue;
                    }

                    if to_decimal(&candidate.peer) > to_decimal(&net.peer) {
                        candidate = net.clone();
                        continue;
                    } else {
                        continue;
                    }
                }
            }
        }
        if candidate.peer != "0" {
            Ok(candidate.peer)
        } else {
            Err(format!("No route"))
        }
    }

    /**
     * This function apply aggregate mechanism to the routing table
     * return Some(Network) if successfully aggregate something,
     * None if nothing gets aggregated.
     */
    fn aggregate(&mut self, network: Network) -> Option<Network> {
        match (0..self.table.len())
            .into_iter()
            .find(|ind| Table::is_aggregable(&self.table[*ind], &network))
        {
            Some(ind) => {
                let net = self.table.remove(ind);
                let new_netmask = to_ipv4(to_decimal(&net.netmask) << 1);
                let new_net_prefix = to_ipv4(apply_mask(&net.network, &new_netmask));
                let aggregated_net = Network::new(
                    net.peer,
                    new_net_prefix,
                    new_netmask,
                    net.localpref,
                    net.selfOrigin,
                    net.ASPath,
                    net.origin,
                );
                Some(aggregated_net)
            }
            None => None,
        }
    }

    /**
     * This function checks if the given networks are aggregable
     */
    fn is_aggregable(net1: &Network, net2: &Network) -> bool {
        // Check if netmasks are same
        if net1.netmask != net2.netmask {
            return false;
        }

        // Check if peers are same
        if net1.peer != net2.peer {
            return false;
        }

        // Check if localprefs are same
        if net1.localpref != net2.localpref {
            return false;
        }

        // Check if AS paths are same
        if net1.ASPath != net2.ASPath {
            return false;
        }

        if net1.selfOrigin != net2.selfOrigin {
            return false;
        }

        // Check if origins are same
        if net1.origin != net2.origin {
            return false;
        }

        // Check if these two networks are numerically adjacent
        if apply_mask_prefix(&net1.network, &net1.netmask)
            .abs_diff(apply_mask_prefix(&net2.network, &net2.netmask))
            != 1
        {
            return false;
        }

        true
    }
}
