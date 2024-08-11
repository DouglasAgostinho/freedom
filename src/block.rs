
use sha2::{Digest, Sha512};
use serde::{Deserialize, Serialize};   
use std::time::{SystemTime,UNIX_EPOCH};

//to use in String based variables
const EMPTY_STRING: String = String::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct NetWorkMessage {
    pub version:    String,
    pub time:       String,
    pub message:    String,
    pub peers:      Vec<Peers>,
    pub code:       String,        
}

impl NetWorkMessage {

    pub fn new() -> NetWorkMessage{
        NetWorkMessage{
            version: EMPTY_STRING,
            time: EMPTY_STRING,
            message: EMPTY_STRING,
            peers: Vec::from([Peers::new()]),
            code: EMPTY_STRING,
        }

    }
    
}

#[derive(Debug, Clone)]
pub struct Block {        
    pub message:    Vec<[String; 3]>,
    pub msg_done:    Vec<[String; 3]>,
}
impl Block {

    ///Insert function works to keep the blocks updated and organized
    pub fn update (&mut self,msg: [String; 3]) {

        //println!(" !!!!! Msg=>{:?}", msg);

        if !self.msg_done.contains(&msg){ //Check if message was already processed

            //Insert data in the Vector
            self.message.push(msg);

            //Organize data based on creation time (index0)
            self.message.sort_by(|a, b| a[0].cmp(&b[0]));
        }
        else{
            //self.message.retain(|x| x != &msg);
            //println!("Msg=>{:?}", msg);
            if let Some(pos) = self.message.iter().position(|x| x == &msg) {
                self.message.swap_remove(pos);
                //println!("Msg=>{:?}, index=>{}", msg, pos);
            }
        }
    }   

    pub fn mark_message(&mut self, i: usize){

        if !self.msg_done.contains(&self.message[i]){

            self.msg_done.push(self.message[i].clone());
        }
        
    }   
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Peers {
    pub address: String,
    pub models: Vec<String>,    
}
impl Peers{

    pub fn new() -> Peers{
        Peers{
            address: EMPTY_STRING,
            models: Vec::from([EMPTY_STRING]),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    //pub address: String,
    pub known_peers: Vec<Peers>,
}
impl Node {

    ///Node address generation
    pub fn _gen_address(&self) -> String{

        let mut hasher = Sha512::new();
    
        // Write input data
        hasher.update(self.get_time_ns());
    
        // Read hash digest and consume hasher
        let result = hasher.finalize();
    
        // Convert the byte array to a hexadecimal string
        result.iter().map(|byte| format!("{:02x}", byte)).collect()
    }

    ///Function responsible to get and return current time in nanoseconds
    pub fn get_time_ns(&self) -> String{

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
        
        since_the_epoch.as_nanos().to_string()
    }

    pub fn insert_peer (&mut self, peer: Peers) {
        //Insert data in the Vector
        if !self.known_peers.contains(&peer){
            self.known_peers.push(peer);
        }
    }      
    
}

