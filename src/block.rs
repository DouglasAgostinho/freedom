
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

//#[derive(Clone)]
pub struct Block {        
    pub message:    Vec<[String; 3]>,
}
impl Block {

    ///Insert function works to keep the blocks updated and organized
    pub fn insert (&mut self,msg: [String; 3]) {

        //Insert data in the Vector
        self.message.push(msg);

        //Organize data based on creation time (index0)
        self.message.sort_by(|a, b| a[0].cmp(&b[0]));
    }       
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    
}

