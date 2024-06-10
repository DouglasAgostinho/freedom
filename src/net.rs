pub mod network{

    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::io::{self,Read, Write, Error};

    fn handle_client(mut stream: TcpStream) -> Result<(), Error>{

        println!("Incoming connection from {}", stream.peer_addr()?);        
        let mut buf = [0; 512];

        loop {
            
            let bytes_read = stream.read(&mut buf)?;
            if bytes_read == 0 {return Ok(())}
            stream.write(&buf[..bytes_read])?;
        }
    }

    pub fn net_init(){

        let listener = TcpListener::bind("0.0.0.0:8888").expect("Could not bind");

        for stream in listener.incoming(){
            match stream {
                Err(e) => println!("Error found {e}"),
                Ok(stream) => {
                    thread::spawn(move || {
                        handle_client(stream).unwrap_or_else(|error| println!("Error {:?}", error));
                    });
                }
            }
        }
    }


    pub fn client(message: &str, address: &str)-> io::Result<()> {
        //use std::net::TcpStream;
        //use std::io::{self, Write, Read};

        // Connect to the server
        let mut stream = TcpStream::connect(address)?;

        // Send data to the server
        //let message = "Hello, server!";
        stream.write_all(message.as_bytes())?;

        // Receive data from the server
        let mut buffer = [0; 1024];
        //let bytes_read = stream.read(&mut buffer)?;
        //let received = String::from_utf8_lossy(&buffer[..bytes_read]);
        //println!("Received: {}", received);

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Connection closed by server");
                    break;
                },
                Ok(n) => {
                    //let received = str::from_utf8(&buffer[0..n]).expect("Failed to parse message");
                    //let received = String::from_utf8_lossy(&buffer[0..n]);
                    //println!("Received: {}", received);
                    //print!("{}", received);
                    //write!(io::stdout(), "{}", received).expect("msg");
                    print!("{}", String::from_utf8_lossy(&buffer[0..n]));
                    io::stdout().flush()?;  // Ensure immediate output
                    //println!("{}", String::from_utf8_lossy(&buffer[0..n]));
                },
                Err(e) => {
                    println!("Failed to receive message: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
    
}


/*

pub fn client(message: &str, address: &str)-> io::Result<()> {
        //use std::net::TcpStream;
        //use std::io::{self, Write, Read};

        // Connect to the server
        let mut stream = TcpStream::connect(address)?;

        // Send data to the server
        //let message = "Hello, server!";
        stream.write_all(message.as_bytes())?;

        // Receive data from the server
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer)?;
        let received = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received: {}", received);

        Ok(())
    }    

*/