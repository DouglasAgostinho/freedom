
use sha2::{Digest, Sha512};
use std::time::{SystemTime,UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Block {        
    pub message:    Vec<[String; 3]>,    
}
impl Block {

    //pub fn add (&mut self, time: String, addr: String, msg: String) {
    pub fn insert (&mut self,msg: [String; 3]) {

        self.message.push(msg);

        self.message.sort_by(|a, b| a[0].cmp(&b[0]));
    }       
}


pub struct Node {
    pub address: String,
}
impl Node {

    pub fn gen_address(&self) -> String{

        let mut hasher = Sha512::new();
    
        // Write input data
        hasher.update(self.get_time_ns());
    
        // Read hash digest and consume hasher
        let result = hasher.finalize();
    
        //let hs = String::from("{result}");


        // Convert the byte array to a hexadecimal string
        //let hash_string: String = result.iter().map(|byte| format!("{:02x}", byte)).collect();
        result.iter().map(|byte| format!("{:02x}", byte)).collect()


        //String::from("{hasher.finalize()}")
        

        // Print the hash as a hexadecimal string
        //println!("Hash: {:x}", result);

    }

    pub fn get_time_ns(&self) -> String{

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
        
        since_the_epoch.as_nanos().to_string()
    }
    
}


/* 
pub struct Letter {        
    pub message:    Vec<[String; 3]>,
    
}
impl Letter {

    //pub fn add (&mut self, time: String, addr: String, msg: String) {
    pub fn add (&mut self,msg: [String; 3]) {

        self.message.push(msg);

        self.message.sort_by(|a, b| a[0].cmp(&b[0]));
    }
    
}

pub mod block{

    
}*/