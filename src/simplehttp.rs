use std::{collections::HashMap, io::{Read, Write}, net::TcpStream};

use native_tls::TlsConnector;

pub fn extract_json_value(json_str: &str, key: &str) -> Option<String> {

    let key_pattern = format!("\"{}\":\"", key);

    if let Some(start) = json_str.find(&key_pattern) {

        let token_start = start + key_pattern.len();

        if let Some(end) = json_str[token_start..].find('\"') {
            return Some(json_str[token_start..token_start + end].to_string());
        }
    }

    None
}
pub struct SimpleHTTP{
    pub host:String
}
impl SimpleHTTP{
    pub fn new(host:&str)->Self{
       

    Self{host: host.to_string()}
    }
   
    pub fn do_https_request(&mut self,path:&str,method:&str,headers:Option<HashMap<&str,&str>>,content:Option<&[u8]>)->Result<Vec<u8>,&'static str>{
        let mut headers_string:String = String::new();
        if let Some(heds) = headers{
        for (key,value) in heds{
            headers_string += format!("\r\n{}: {}",key,value).as_str();
        }
    }
       
        let mut content_length:usize = 0;
        if content.is_some(){
            content_length = content.unwrap().len();
        }
        let request_data_string:String = format!("{} {} HTTP/1.1\r\nHost: {}{}\r\nContent-Length:{}\r\nConnection: close\r\n\r\n",method,path,self.host,headers_string,content_length);
        let mut request_data:Vec<u8> =  request_data_string.as_bytes().to_vec();
        if content.is_some(){
            request_data.extend_from_slice(content.unwrap());
        }
      
        let connector = TlsConnector::new().unwrap();
        let stream = TcpStream::connect(format!("{}:443",self.host)).expect("Could not connect to the remote server!");
        let mut stream = connector.connect(format!("{}",self.host).as_str(), stream).unwrap();
        let mut is_error_writing:bool = false;
        stream.write_all(&request_data).unwrap_or_else(|_|{
          is_error_writing = true;
        });
       
        

        if is_error_writing{return Err("Could not write to TLSStream!");}
        let mut res: Vec<u8> = vec![];
        let size_read = stream.read_to_end(&mut res).unwrap_or_else(|_|{
           usize::MAX
        });
        if size_read == usize::MAX{
            Err("Could not read from TLSStream!")
        }else{
            let temp = &res[..size_read];
            Ok(temp.to_vec())
        }

    }
}