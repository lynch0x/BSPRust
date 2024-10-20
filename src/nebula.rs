use std::collections::HashMap;

use url::form_urlencoded;
use base64::decode;

use crate::simplehttp::{extract_json_value, SimpleHTTP};
pub const USASERVERS:[&str;4] = ["AU","NZ","CA","US"];
pub struct LoginResult{
    pub is_logged_in:bool,
    pub access_token:String,
    pub refresh_token:String,
    pub profile_id:String,
    pub server:String
}

impl Default for LoginResult{
    fn default()->Self{
        LoginResult{
            is_logged_in:false,
            access_token: String::new(),
            refresh_token: String::new(),
            profile_id: String::new(),
            server: String::new()
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


fn get_tokens_from_json(json_str: &str) -> Result<(String, String), &'static str> {
    let access_token = extract_json_value(json_str, "access_token").ok_or("Missing access_token")?;

    let refresh_token = extract_json_value(json_str, "refresh_token").ok_or("Missing refresh_token")?;

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
    

    let mut headers: HashMap<&str, &str> = HashMap::new();
    let temp =format!("Bearer {}",token);
    headers.insert("Authorization", temp.as_str());
    let mut http = SimpleHTTP::new(format!("{}.mspapis.com",endpoint_region).as_str());
    let res = http.do_https_request(format!("/profileidentity/v1/logins/{}/profiles",login_id).as_str(), "GET", Some(headers), None).unwrap_or_else(|error|{
        eprintln!("ERROR {}",error);
        return vec![];
    });
    let response = String::from_utf8_lossy(&res).to_string();
  
    let json = get_json_string(response).expect("Request failed!");
    let profile_id = extract_json_value(json.as_str(), "id").ok_or("Could not get profileid!").unwrap();
    return profile_id;
}
fn login_to_nebula(server:&String,username:&String, password:&String)->LoginResult{
    let endpoint_region:&str;
    if USASERVERS.contains(&server.as_str())  {endpoint_region = "us";} else {endpoint_region="eu";}
  
    let urlencodedusername:String= form_urlencoded::byte_serialize(username.as_bytes()).collect();
    let urlencodedpassword:String= form_urlencoded::byte_serialize(password.as_bytes()).collect();
   
    let formencodedcontent:String = format!("client_id=unity.client&grant_type=password&username={}{}&password={}&acr_values=gameId%3Aywru",server.to_owned()+"|",urlencodedusername,urlencodedpassword);
  
    let mut http = SimpleHTTP::new(format!("{}-secure.mspapis.com",endpoint_region).as_str());
    let mut headers: HashMap<&str, &str> = HashMap::new();
    headers.insert("Content-Type", "application/x-www-form-urlencoded");
    let res = http.do_https_request("/loginidentity/connect/token", "POST", Some(headers), Some(formencodedcontent.as_bytes())).unwrap_or_else(|error|{
        eprintln!("ERROR {}",error);
        return vec![];
    });

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

    let formencodedcontent:String = format!("grant_type=refresh_token&refresh_token={}&acr_values=gameId%3aywru%20profileId%3a{}",data.refresh_token,data.profile_id);

    let mut http = SimpleHTTP::new(format!("{}-secure.mspapis.com",endpoint_region).as_str());
    let mut headers: HashMap<&str, &str> = HashMap::new();
    headers.insert("Content-Type", "application/x-www-form-urlencoded");
    headers.insert("Authorization", "Basic dW5pdHkuY2xpZW50OnNlY3JldA==");
    let res = http.do_https_request("/loginidentity/connect/token", "POST", Some(headers), Some(formencodedcontent.as_bytes())).unwrap_or_else(|error|{
        eprintln!("ERROR {}",error);
        return vec![];
    });
    

    
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
    result.server = server;
    result
  }