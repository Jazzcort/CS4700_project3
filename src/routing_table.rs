use crate::ipv4::{self, apply_mask, apply_mask_prefix, netmask_digit, to_decimal, to_ipv4};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Origin {
    Igp = 3,
    Egp = 2,
    Unk = 1,
}

pub struct Network {
    peer: String,
    net_prefix: String,
    netmask: String,
    localpref: i32,
    self_origin: bool,
    as_path: Vec<i32>,
    origin: Origin,
}

impl Network {
    pub fn new(
        peer: String,
        net_prefix: String,
        netmask: String,
        localpref: i32,
        self_origin: bool,
        as_path: Vec<i32>,
        origin: Origin,
    ) -> Self {
        Network {
            peer,
            net_prefix,
            netmask,
            localpref,
            self_origin,
            as_path,
            origin,
        }
    }
}

pub struct Table {
    table: Vec<Network>,
}

impl Table {
    pub fn new() -> Self {
        Table { table: vec![] }
    }

    pub fn update(&mut self, new_net: Network) {}

    fn aggregate(&mut self, network: Network) -> Option<Network> {

        match (0..self.table.len()).into_iter().find(|ind| Table::is_aggregable(&self.table[*ind], &network)) {
            Some(ind) => {
                let net =  self.table.remove(ind);
                let new_netmask = to_ipv4(to_decimal(&net.netmask) << 1);
                let new_net_prefix = to_ipv4(apply_mask(&net.net_prefix, &new_netmask));
                let aggregated_net = Network::new(net.peer, new_net_prefix, new_netmask, net.localpref, net.self_origin, net.as_path, net.origin);

            },
            None => {}
        }

        // for net in self.table.iter().enumerate() {
        //     if Table::is_aggregable(net, &network) {
        //         found = Some()
        //     }
        // }
        
        None
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
        if net1.as_path != net2.as_path {
            return false;
        }

        if net1.self_origin != net2.self_origin {
            return false;
        }

        // Check if origins are same
        if net1.origin != net2.origin {
            return false;
        }

        // Check if these two networks are numerically adjacent
        if apply_mask_prefix(&net1.net_prefix, &net1.netmask)
            .abs_diff(apply_mask_prefix(&net2.net_prefix, &net2.netmask))
            != 1
        {
            return false;
        }

        true
    }
}
