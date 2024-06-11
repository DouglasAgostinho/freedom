pub mod network{

    use std::net::{TcpListener, TcpStream};
    use std::thread;
    //use std::io::{self,Read, Write, Error};
    use std::io::{self,Read, Write};

    //use sha2::digest::consts::False;


    fn handle_message(message: &String, mode: &str) -> bool{
        //Function to treat incoming / outgoing messages        
        match mode {

            "send" => {false},

            "receive" => {
                
                match message.trim() {                    

                    "[!]_stream_[!]" => true, //(true, "send_llm_output".to_string()),

                    "[!]_blocks_[!]" => false, //to_do will be used to return message block

                    _ => {
                        println!("Received: {}", message);
                        //(false, EMPTY_STRING) //to_do Will return decrypted message
                        false
                    },
                }
            },

            "test" => {
                println!("Received: {}", message);
                //(false, EMPTY_STRING)
                false
            },

            _ => false,//(false, EMPTY_STRING),
        }
    }


    fn handle_client(mut stream: TcpStream) {
        //fn handle_client(mut stream: TcpStream) -> Result<(), Error>{

        let income_addr = stream.peer_addr().expect("Error");
        //println!("Incoming connection from {}", stream.peer_addr()?);   
        println!("Incoming connection from {}", income_addr);
        let mut buf = [0; 512];

        loop {
                        
            let bytes_read = stream.read(&mut buf).expect("Error");            
            if bytes_read == 0 {break}
            
            let received = String::from_utf8_lossy(&buf[..bytes_read]);

            if handle_message(&received.to_string(), "receive") {                                
                                                     
                stream.write_all("ready_to_receive".as_bytes()).expect("error");
                // Receive data continuously from the server
                let mut buffer = [0; 1024];                    
                loop {
                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Connection closed by server");
                            break;
                        },
                        Ok(n) => {
                            let msg = String::from_utf8_lossy(&buffer[0..n]);
                            print!("{}", msg);      //Uses print! to not insert /n after each received data                                                            
                            io::stdout().flush().expect("error");  // Ensure immediate output
                        },
                        Err(e) => {
                            println!("Failed to receive message: {}", e);
                            break;
                        }
                    }
                }                    
            } 
            else {
                println!("Connection closed by server");
                break;                
            }           
        }
    }


    pub fn net_init(){

        let listener = TcpListener::bind("0.0.0.0:6886").expect("Could not bind");
        println!("Server initialized...");

        for stream in listener.incoming(){
            match stream {
                Err(e) => println!("Error found {e}"),
                Ok(stream) => {
                    thread::spawn(move || {
                        //handle_client(stream).unwrap_or_else(|error| println!("Error {:?}", error));
                        handle_client(stream);
                    });
                }
            }
        }
    }


    pub fn client(message: &str, address: &str, mode: &str)-> io::Result<()> {
        match mode {
            "simple" => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;
                // Send data to the server
                stream.write_all(message.as_bytes())?;
                Ok(())
            },
            "serialized" => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;

                //println!("Pre Serde {:?}", message);

                let serialized = serde_json::to_string(&message)?;
                stream.write_all(serialized.as_bytes())?;
                Ok(())
            },
            "test" => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;
                stream.write_all(message.as_bytes())?;
                Ok(())
            },
            _ => Ok(()),
        }
    }
}