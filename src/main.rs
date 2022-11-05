use mpd::{Client,Song};
use getopts::Occur;
use args::{Args,ArgsError};
use args::validations::{Order, OrderValidation, Validation};

use rand::thread_rng;
use rand::seq::SliceRandom;

use std::cmp::{max,min};
use std::ops::Range;

struct HostnameValidation;

impl Validation for HostnameValidation {
    type T = String;

    fn error(&self, value: &String) -> ArgsError {
        ArgsError::new("hostname invalid", 
            &format!("{} is not a valid hostname", value))
    }

    fn is_valid(&self, value: &String) -> bool {
        hostname_validator::is_valid(value)
    }
}

fn main() {
    let url = match parse() {
        Err(error) => panic!("{:?}", error), 
        Ok(Some(s)) => s,
        Ok(None) => return
    };

    let mut conn = Client::connect(url).expect("failed to connect to mpd");
    

    let queue = match conn.queue() {
        Err(err) => panic!("{:?}",err),
        Ok(q) => q
    };
    
    let mut albums: Vec<&str> = Vec::new();    
    let mut last: Option<&String> = None;

    for s in &queue {
        let album = s.tags.get("Album");
        //TODO: handle blank albums better
        if !album.eq(&last) {
            if let Some(a) = album {
                albums.push(a);
            }
        }
        last = album;
    }
    
    let mut rng = thread_rng();

    albums.shuffle(&mut rng);
    
    //TODO: this shuffle method assumes the playlist is already sorted by album
    for a in albums {
        let queue = match conn.queue() {
            Err(err) => panic!("{:?}",err),
            Ok(q) => q
        };
        if let Err(err) = conn.shift(get_album_bounds(a, &queue).unwrap(),0) {
            panic!("{:?}",err)
        }
    }

}

fn get_album_bounds(album: &str, queue: &[Song]) -> Option<Range<u32>> {
    let mut found = false;
    let mut start = queue.len();
    let mut end   = 0;
    for (i, s) in queue.iter().enumerate() {
        if let Some(a) = s.tags.get("Album") {
            if a.eq(album) {
                found = true;
                start = min(start, i);
                end = max(end, i+1);
            }
        }
    }
    if found { Some(start.try_into().unwrap()..end.try_into().unwrap()) } else { None }

}

fn parse() -> Result<Option<String>, ArgsError> { 
    let mut args = Args::new("shuffle-albums", "shuffles albums in mpd play queue");
    args.flag("h","help", "print the help message");
    
    args.option("H", 
        "host", 
        "the hostname to connect to (default 127.0.0.1)", 
        "HOSTNAME", 
        Occur::Optional, 
        Some(String::from("127.0.0.1")));
    
    args.option("p", 
        "port", 
        "the port to connect to mpd on (default 6600)", 
        "PORT", 
        Occur::Optional, 
        Some(String::from("6600")));

    args.parse_from_cli()?;
    let help = args.value_of("help")?;
    if help {
        eprintln!("{}",args.full_usage());
        return Ok(None);
    }
    
    let port_valid = Box::new(OrderValidation::new(Order::LessThan, 65536u32));
    let port = args.validated_value_of("port", &[port_valid])?;
    
    let host_valid = Box::new(HostnameValidation);
    let host = args.validated_value_of("host", &[host_valid])?;

    Ok(Some(format!("{}:{}",host,port)))
}
