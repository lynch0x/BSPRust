use std::{io::{Read, Write}, net::TcpStream};

use native_tls::TlsConnector;
use url::form_urlencoded;

const USASERVERS:[&str;4] = ["AU","NZ","CA","US"];
pub struct LoginResult{
    pub is_logged_in:bool,
    pub access_token:String,
    pub refresh_token:String
}

impl Default for LoginResult{
    fn default()->Self{
        LoginResult{
            is_logged_in:false,
            access_token: String::new(),
            refresh_token: String::new()
        }
    }
}
//To jest kurwa z chatgpt
fn get_json_string(string: String) -> Result<String, ()> {
    if let Some(start) = string.find("{") {
        // Find the end of the JSON response
        if let Some(end) = string.rfind("}") {
            // Slice the string to extract the JSON content and return it
            let json_content = &string[start..=end];
            return Ok(json_content.to_string()); // Return the JSON part as a String
        }
    }

    Err(()) // Return error if no valid JSON was found
}
fn extract_token(json_str: &str, key: &str) -> Option<String> {
    // Create the key format with quotes, e.g., "\"access_token\":"
    let key_pattern = format!("\"{}\":\"", key);

    // Find the start of the key
    if let Some(start) = json_str.find(&key_pattern) {
        // Find the start of the actual token value
        let token_start = start + key_pattern.len();

        // Find the end of the token (the next quotation mark)
        if let Some(end) = json_str[token_start..].find('\"') {
            // Extract and return the token
            return Some(json_str[token_start..token_start + end].to_string());
        }
    }

    // Return None if the key or value wasn't found
    None
}

fn get_tokens_from_json(json_str: &str) -> Result<(String, String), &'static str> {
    // Try to extract the "access_token"
    let access_token = extract_token(json_str, "access_token").ok_or("Missing access_token")?;

    // Try to extract the "refresh_token"
    let refresh_token = extract_token(json_str, "refresh_token").ok_or("Missing refresh_token")?;

    // Return the tokens as a tuple
    Ok((access_token, refresh_token))
}

pub fn login_to_nebula(server:String,username:String, password:String)->LoginResult{
    let endpoint_region:&str;
    if USASERVERS.contains(&server.as_str())  {endpoint_region = "us";} else {endpoint_region="eu";}
    let mut res = vec![];
{
    let connector = TlsConnector::new().unwrap();
    let stream = TcpStream::connect(format!("{}-secure.mspapis.com:443",endpoint_region)).expect("Could not connect to BSP server [TCP]");
    let mut stream = connector.connect(format!("{}-secure.mspapis.com",endpoint_region).as_str(), stream).expect("Could not connect to BSP server [TLS]");

    let urlencodedusername:String= form_urlencoded::byte_serialize(username.as_bytes()).collect();
    let urlencodedpassword:String= form_urlencoded::byte_serialize(password.as_bytes()).collect();
   
    let formencodedcontent:String = format!("client_id=unity.client&grant_type=password&username={}{}&password={}&acr_values=gameId%3Aywru",server+"|",urlencodedusername,urlencodedpassword);
    let formated =format!("POST /loginidentity/connect/token HTTP/1.1\r\nHost: {}-secure.mspapis.com\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length:{}\r\nConnection: close\r\n\r\n{}",endpoint_region,formencodedcontent.len(),formencodedcontent);
  
    stream.write_all(formated.as_bytes()).expect("Could not write to BSP server!");
    stream.read_to_end(&mut res).expect("Could not read from BSP server!");
    
}
    let response = String::from_utf8_lossy(&res).to_string();
    let json = get_json_string(response).expect("Request failed!");
   
        let tokens = get_tokens_from_json(json.as_str());
        if tokens.is_ok(){
            let unwraped = tokens.unwrap();
            let mut result:LoginResult = LoginResult::default();
            result.is_logged_in=true;
            result.access_token =unwraped.0;
            result.refresh_token = unwraped.1;
            return result;
        }
    return LoginResult::default();
  }