

use std::{f64::INFINITY,  io::prelude::*, thread, time::Duration};
mod nebula;

fn ask(msg:&str)->String{
  use std::io::{stdin,stdout};
  print!("{}",msg);
  let mut s:String = String::new();
 let _ = stdout().flush();
 stdin().read_line(&mut s).expect("You didn't enter a valid string!");
 if let Some('\n')=s.chars().next_back() {
  s.pop();
}
if let Some('\r')=s.chars().next_back() {
  s.pop();
}
s
}
fn main(){
  
  let server:String = ask("Server: ");
  
  let username:String = ask("Username: ");
  let password:String = ask("Password: ");

  let result = nebula::login(server,username,password);
  match result.is_logged_in{
    true=> {
      println!("Logged in, ProfileId: {}\nToken: {}",result.profile_id,result.access_token);
      
    },
    false => println!("Could not login in!")
  }

  thread::sleep(Duration::from_millis(INFINITY as u64));
}
