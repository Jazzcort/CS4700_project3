use std::{any::Any, net::UdpSocket};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Number, Value};

use crate::routing_table::Table;


// enum Type {
//     Update,
//     Withdraw,
//     Data,

// }
#[derive(Debug)]
pub enum NeighborType {
    Peer,
    Cust,
    Prov
}


#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,
    dst: String,
    r#type: String,
    msg: Value
}
pub struct Router {
    receiver: UdpSocket,
    pub sender: HashMap<String, (NeighborType, String)>,
    table: Table
}

impl Router {
    pub fn new(addr: String) -> Result<Self, String> {
        let router = Router {
            receiver: UdpSocket::bind(addr).map_err(|e|format!("{e} -> faild to bind host socket"))?,
            sender: HashMap::new(),
            table: Table::new()
        };
        // dbg!(router.receiver.set_read_timeout(Some(Duration::from_millis(500))).unwrap());

        Ok(router)
    }

    fn read_to_string(buf: &mut [u8]) -> Result<String, String> {
        for ind in 0..buf.len() {
            if buf[ind] == 0 {
                return Ok(String::from_utf8_lossy(&buf[..ind]).to_string())
            }
        }
        return Err(format!("Data incomplete"));
    }

    pub fn register_neighbor(&mut self, ip: &str, port: &str, neighbor_type: NeighborType) {
        self.sender.insert(ip.to_string(), (neighbor_type, format!("127.0.0.1:{port}")));
    }

    pub fn keep_listen(&mut self) {
        let mut buf: [u8; 2048] = [0; 2048];
        
        loop {

            self.receiver.recv(&mut buf).unwrap();
            let res = Router::read_to_string(&mut buf).unwrap();

            println!("{}", &res);
            // let a = serde_json::from_slice(res.as_bytes());
            dbg!(res.len());
            let mut v: Message = serde_json::from_str(&res).expect("failed!!!");

            // println!("{}", v.msg["ASPath"]);
            if let Value::Array(mut a) = v.msg["ASPath"].clone() {
                a.push(json!(2));
                println!("{:?}", a);
                v.msg["ASPath"] = json!(a);
            }
            match v.r#type.as_str() {
                 "update"=> {
                    let src = v.src.clone();
                    dbg!(&src);
                    let (tmp, addr) = self.sender.get_mut(&src).unwrap();
                    self.receiver.send_to(json!(v).to_string().as_bytes(), "127.0.0.1:7878".to_string()).unwrap();
                 },

                _ => {}

            }

            println!("{:?}", v);

            // let j = json!({"a": "123"});
        }

    }
}