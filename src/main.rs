extern crate pnet;

use quicli::prelude::*;
use structopt::StructOpt;

use pnet::datalink::Channel::Ethernet;
use pnet::datalink::ChannelType;
use pnet::datalink::Config;
use pnet::datalink::{self, DataLinkReceiver, NetworkInterface};
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::{Packet, PacketSize};

/// Peek at the packets on a network interface
#[derive(Debug, StructOpt)]
struct Cli {
    /// Packets to recieve
    #[structopt(long = "npkts", short = "n", default_value = "0")]
    npkts: usize,

    /// The interface to peek
    iface_name: String,

    /// The EtherType to fileter on
    #[structopt(long = "ethtype", short = "t", default_value = "0")]
    eth_type: u16,

    // -v Warn, -vv Info, -vvv Debug
    #[structopt(flatten)]
    verbosity: Verbosity,
}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger("peek_iface")?;

    let match_iface_name = |iface: &NetworkInterface| iface.name == args.iface_name;

    let iface = datalink::interfaces()
        .into_iter()
        .filter(match_iface_name)
        .next()
        .unwrap();

    info!("iface_name: {} iface: {:#?}", args.iface_name, iface);
    info!("reading npkts: {}", args.npkts);

    let mut channel_config = Config::default().clone();
    if args.eth_type > 0 {
        channel_config.channel_type = ChannelType::Layer3(args.eth_type);
    }

    info!("EtherType: {}", args.eth_type);
    debug!("channel type: {:?}", ChannelType::Layer3(args.eth_type));
    debug!("Channel config: {:#?}", channel_config);

    let (_, mut rx) = match datalink::channel(&iface, channel_config) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };

    let recv_pkt = |rx: &mut Box<dyn DataLinkReceiver>| match rx.next() {
        Ok(packet) => {
            let packet = EthernetPacket::new(packet).unwrap();
            println!(
                "\n{} {} {} {}",
                packet.get_source(),
                packet.get_destination(),
                packet.get_ethertype(),
                PacketSize::packet_size(&packet)
            );

            print!("\t");
            for b in packet.payload() {
                if 31u8 < *b && *b <= 125u8 {
                    print!("{}", *b as char);
                } else {
                    print!(" {:#02x} ", b);
                }
            }
            println!();
        }
        Err(e) => {
            panic!("An error occurred while reading: {}", e);
        }
    };

    match args.npkts {
        0 => loop {
            recv_pkt(&mut rx);
        },
        _ => {
            for _ in 0..args.npkts {
                recv_pkt(&mut rx);
            }
        }
    }

    Ok(())
}
