use std::{io::{Read, Write}, net::TcpStream};

use native_tls::TlsConnector;
use url::form_urlencoded;
use base64::decode;
const USASERVERS:[&str;4] = ["AU","NZ","CA","US"];
pub struct LoginResult{
    pub is_logged_in:bool,
    pub access_token:String,
    pub refresh_token:String,
    pub profile_id:String
}

impl Default for LoginResult{
    fn default()->Self{
        LoginResult{
            is_logged_in:false,
            access_token: String::new(),
            refresh_token: String::new(),
            profile_id: String::new()
        }
    }
}
fn get_json_string(string: String) -> Result<String, ()> {
    if let Some(start) = string.find("{") {
        if let Some(end) = string.rfind("}") {
            
            let json_content = &string[start..=end];
            return Ok(json_content.to_string()); 
        }
    }

    Err(())
}
fn extract_token(json_str: &str, key: &str) -> Option<String> {

    let key_pattern = format!("\"{}\":\"", key);

    if let Some(start) = json_str.find(&key_pattern) {

        let token_start = start + key_pattern.len();

        if let Some(end) = json_str[token_start..].find('\"') {
            return Some(json_str[token_start..token_start + end].to_string());
        }
    }

    None
}

fn get_tokens_from_json(json_str: &str) -> Result<(String, String), &'static str> {
    let access_token = extract_token(json_str, "access_token").ok_or("Missing access_token")?;

    let refresh_token = extract_token(json_str, "refresh_token").ok_or("Missing refresh_token")?;

    Ok((access_token, refresh_token))
}
fn login_id_from_access_token(token:&String)->String{
 let split:Vec<&str> = token.split('.').collect();
 let text = split[1];
 let mut fixed_base64 = text.replace("-", "+").replace("_","/");
 let mut i:usize = 0 ;
 loop{
    if i >= fixed_base64.len() % 4{break;}
 fixed_base64 += "=";
 i+=1;
 }
 let decoded_base64_bytes = decode(fixed_base64).expect("Could not decode base64 string!");
 let decoded_base64_string = String::from_utf8(decoded_base64_bytes).unwrap();
 let split:Vec<&str> = decoded_base64_string.split('"').collect();

return split[19].to_string();
}
fn get_profile_id_from_token(server:&String,token:&String)->String{

    let login_id = login_id_from_access_token(token);
    let endpoint_region:&str;
    if USASERVERS.contains(&server.as_str())  {endpoint_region = "us";} else {endpoint_region="eu";}
    let mut res = vec![];

    let connector = TlsConnector::new().unwrap();
    let stream = TcpStream::connect(format!("{}.mspapis.com:443",endpoint_region)).expect("Could not connect to BSP server [TCP]");
    let mut stream = connector.connect(format!("{}.mspapis.com",endpoint_region).as_str(), stream).expect("Could not connect to BSP server [TLS]");

    let formated =format!("GET /profileidentity/v1/logins/{}/profiles HTTP/1.1\r\nHost: {}.mspapis.com\r\nAuthorization: Bearer {}\r\nConnection: close\r\n\r\n",login_id,endpoint_region,token);
  
    stream.write_all(formated.as_bytes()).expect("Could not write to BSP server!");
    stream.read_to_end(&mut res).expect("Could not read from BSP server!");
    let response = String::from_utf8_lossy(&res).to_string();
    let json = get_json_string(response).expect("Request failed!");
    let profile_id = extract_token(json.as_str(), "id").ok_or("Could not get profileid!").unwrap();
    return profile_id;
}
fn login_to_nebula(server:&String,username:&String, password:&String)->LoginResult{
    let endpoint_region:&str;
    if USASERVERS.contains(&server.as_str())  {endpoint_region = "us";} else {endpoint_region="eu";}
    let mut res = vec![];
{
    let connector = TlsConnector::new().unwrap();
    let stream = TcpStream::connect(format!("{}-secure.mspapis.com:443",endpoint_region)).expect("Could not connect to BSP server [TCP]");
    let mut stream = connector.connect(format!("{}-secure.mspapis.com",endpoint_region).as_str(), stream).expect("Could not connect to BSP server [TLS]");

    let urlencodedusername:String= form_urlencoded::byte_serialize(username.as_bytes()).collect();
    let urlencodedpassword:String= form_urlencoded::byte_serialize(password.as_bytes()).collect();
   
    let formencodedcontent:String = format!("client_id=unity.client&grant_type=password&username={}{}&password={}&acr_values=gameId%3Aywru",server.to_owned()+"|",urlencodedusername,urlencodedpassword);
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
  fn refresh_token(server:&String,data:&mut LoginResult){
    let endpoint_region:&str;
    if USASERVERS.contains(&server.as_str())  {endpoint_region = "us";} else {endpoint_region="eu";}
    let mut res = vec![];
{
    let connector = TlsConnector::new().unwrap();
    let stream = TcpStream::connect(format!("{}-secure.mspapis.com:443",endpoint_region)).expect("Could not connect to BSP server [TCP]");
    let mut stream = connector.connect(format!("{}-secure.mspapis.com",endpoint_region).as_str(), stream).expect("Could not connect to BSP server [TLS]");

    let formencodedcontent:String = format!("grant_type=refresh_token&refresh_token={}&acr_values=gameId%3aywru%20profileId%3a{}",data.refresh_token,data.profile_id);
    let formated =format!("POST /loginidentity/connect/token HTTP/1.1\r\nHost: {}-secure.mspapis.com\r\nAuthorization: Basic dW5pdHkuY2xpZW50OnNlY3JldA==\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length:{}\r\nConnection: close\r\n\r\n{}",endpoint_region,formencodedcontent.len(),formencodedcontent);
  
    stream.write_all(formated.as_bytes()).expect("Could not write to BSP server!");
    stream.read_to_end(&mut res).expect("Could not read from BSP server!");
    
}
    let response = String::from_utf8_lossy(&res).to_string();
 
    let json = get_json_string(response).expect("Request failed!");
   
        let tokens = get_tokens_from_json(json.as_str());
        if tokens.is_ok(){
            let unwraped  =tokens.unwrap();
           data.access_token = unwraped.0;
            data.refresh_token = unwraped.1;
            
        }else{
            panic!("Token is not ok [refresh_token]");
        }
   
  }
  pub fn login(server:String,username:String,password:String)->LoginResult{
    let mut result = login_to_nebula(&server, &username, &password);
    if !result.is_logged_in{
        return result;
    }
    result.profile_id = get_profile_id_from_token(&server, &result.access_token);
    refresh_token(&server, &mut result);
    result
  }