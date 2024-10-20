use std::{net::TcpStream, time::Duration};

use crate::{nebula::LoginResult, simplehttp::{extract_json_value, SimpleHTTP}};

pub struct SimpleFOX{
    pub login_data:LoginResult,
    pub stream:TcpStream
}
impl SimpleFOX{
    pub fn new(login_data_param:LoginResult)->Self{
        let endpoint_region:&str;
    if crate::nebula::USASERVERS.contains(&login_data_param.server.as_str())  {endpoint_region = "us";} else {endpoint_region="eu";}
        let mut http = SimpleHTTP::new(format!("central-{}-alb.rbpapis.com",endpoint_region).as_str());
        let response = http.do_https_request("/clusterstat/serverinfo", "GET", None, None).unwrap();
        let response_string = String::from_utf8(response).unwrap();
        let ip = extract_json_value(&response_string, "publicIp").unwrap();
        let stream = TcpStream::connect_timeout(&format!("{}:{}",ip,"843").parse().unwrap(), Duration::from_secs(3)).unwrap();
        SimpleFOX{
            login_data: login_data_param,
            stream: stream
        }
    }
}