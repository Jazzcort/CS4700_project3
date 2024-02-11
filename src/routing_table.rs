use crate::ipv4::{self, apply_mask, apply_mask_prefix, netmask_digit, to_decimal, to_ipv4};
use serde::{Deserialize, Serialize};


#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Serialize, Deserialize)]
pub enum Origin {
    IGP = 3,
    EGP = 2,
    UNK = 1,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Network {
    peer: String,
    network: String,
    netmask: String,
    localpref: i32,
    selfOrigin: bool,
    ASPath: Vec<i32>,
    origin: Origin,
}

impl Network {
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

#[derive(Debug, Clone, Serialize)]
pub struct Table {
    table: Vec<Network>,
}

impl Table {
    pub fn new() -> Self {
        Table { table: vec![] }
    }

    pub fn update(&mut self, mut new_net: Network) {
        loop {
            match self.aggregate(new_net.clone()) {
                Some(n) => new_net = n,
                None => break,
            }
        }
        self.table.push(new_net)
    }

    pub fn get_table(&self) -> &Vec<Network> {
        &self.table
    }

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
