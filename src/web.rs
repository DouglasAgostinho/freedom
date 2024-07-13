pub mod services{
    use std::fs;
    use std::net::{TcpListener, TcpStream};
    use std::io::prelude::*;
    use tracing::{info, error, instrument};
    use crate::net::network::NET_BUFFER;

    const WEB_PORT: &str = "8668";

    #[instrument]
    pub fn web_server() {
        //Composing IP address with received port
        let mut addr = String::from("0.0.0.0:");
        addr.push_str(WEB_PORT);

        //Set system to listen
        let listener = match TcpListener::bind(addr){
            Ok(l) => l,
            Err(e) => {
                error!("Error while binding address => {}", e);
                return
            }
        };

        info!("Server initialized...");
        
        //Create a thread for each received connection
        for stream in listener.incoming(){
            match stream {
                Err(e) => error!("Error found 0 {e}"),
                Ok(stream) => {

                    handle_connection(stream);
                    
                }
            }
        }
    }

    #[instrument]
    fn handle_connection(mut stream: TcpStream){

        let mut buf = NET_BUFFER;

        match stream.read(&mut buf){
            Ok(0) => {
                info!("Connection closed by server");
                return;
            },
            Ok(b) => b,
            Err(e) => {
                error!("Error while reading stream => {}", e);
                return;
            }
        };

        let get = b"GET / HTTP/1.1\r\n";

        let (status_line, filename) = 
            if buf.starts_with(get){

                ("HTTP/1.1 200 OK", "index.html")
            
            } else {
              
                ("HTTP/1.1 404 OK", "404.html")   
            };

        let contents = fs::read_to_string(filename).expect("Error file");

        
        let response = format!(

            "{}\r\nContent-Length: {}\r\n\r\n{}", 
            status_line,
            contents.len(),
            contents
        );

        match stream.write_all(response.as_bytes()){
            Ok(s) => s,
            Err(e) => {
                error!("Error while trying to send network message => {}", e);
                return
            }
        }
        stream.flush().expect("Error flush");

        //println!("Received => {}", received);

    }
}