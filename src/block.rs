

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

    
}