extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate pcarp;

use clap::App;
use pcarp::*;
use std::fs::File;
use std::thread;
use std::time::*;

fn main() {
    let args = App::new("pcap_dump")
        .version("0.1")
        .about("Dumps the packets from a pcapng file")
        .args_from_usage(
            "<pcap>  'The pcapng file to read from'
             [verbosity]... -v 'Sets the level of verbosity'",
        ).get_matches();

    // Initialise the logger
    let log_level = log_level_from_int(args.occurrences_of("verbosity"));
    env_logger::Builder::new().filter(None, log_level).init();

    let file = File::open(args.value_of("pcap").unwrap()).unwrap();
    let mut pcap = Pcapng::new(file).unwrap();
    let ts = Instant::now();
    let mut n = 0;
    loop {
        match pcap.next() {
            Ok(Some(pkt)) => {
                n += 1;
                println!("{:?}", pkt.timestamp.unwrap());
            }
            Ok(None) => { /* the block was not a packet */ }
            Err(Error::NotEnoughBytes(e, a)) => {
                info!("Not enough bytes ({}/{}); sleeping and retrying.", a, e);
                thread::sleep(Duration::from_millis(500));
            }
            Err(Error::ZeroBytes) => {
                info!("EOF. Terminating");
                break;
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
        if n % 1000 == 0 {
            let nanos = ts.elapsed().subsec_nanos();
            let bps = n as f64 * 1_000_000_000.0 / nanos as f64;
            info!("Read {} blocks at {} pps", n, bps);
        }
    }
}

pub fn log_level_from_int(n: u64) -> log::LevelFilter {
    match n {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    }
}