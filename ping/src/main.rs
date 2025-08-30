use std::cmp;
use std::net::Ipv4Addr;
use std::sync::{Mutex, Condvar};
use std::thread;
use std::time::Duration;
use std::mem::MaybeUninit;

use clap::Parser;
use windows::Win32::Foundation::*;
use windows::Win32::NetworkManagement::IpHelper::ICMP_ECHO_REPLY;
use windows::Win32::Networking::WinSock::*;
use windows::Win32::System::Console::*;

mod ping;

static mut STATS: Mutex<PingStats> = Mutex::new(PingStats::new());
static mut TGT_IP_SET: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());
static mut TGT_IP: Mutex<MaybeUninit<Ipv4Addr>> = Mutex::new(MaybeUninit::uninit());

#[derive(Parser)]
pub struct CliArgs {
    /// Ping the specified host until stopped.
    /// To see statistics and continue - type Control-Break;
    /// To stop - type Control-C.
    #[arg(short = 't', verbatim_doc_comment)]
    until_stopped: bool,
    /// Resolve addresses to hostnames.
    #[arg(short = 'a', verbatim_doc_comment)]
    resolve_addresses: bool,
    /// Number of echo requests to send.
    #[arg(short = 'n', default_value_t = 4, verbatim_doc_comment)]
    count: u32,
    /// Send buffer size.
    #[arg(short = 'l', default_value_t = 32, verbatim_doc_comment)]
    size: u16,
    /// Set Don't Fragment flag in packet.
    #[arg(short = 'f', verbatim_doc_comment)]
    dont_fragment: bool,
    /// Time To Live.
    #[arg(short = 'i', verbatim_doc_comment)]
    ttl: Option<u8>,
    /// Timeout in milliseconds to wait for each reply.
    #[arg(short = 'w', default_value_t = 4000, verbatim_doc_comment)]
    timeout: u32,
    /// Source address to use.
    #[arg(short = 'S', verbatim_doc_comment)]
    srcaddr: Option<Ipv4Addr>,
    /// The target host to ping.
    #[arg(verbatim_doc_comment)]
    target_name: String,
}

pub fn main() -> anyhow::Result<()> {
    ping::init_winsock()?;

    let args = CliArgs::parse();

    let (tgt_ip, tgt_hostname) = get_tgt_ip_and_hostname(&args)?;
    {
        // Set the target IP address for use by the console handler (if ever called).
        unsafe {
            let mut lock = TGT_IP_SET.0.lock().unwrap();
            TGT_IP.lock().unwrap().write(tgt_ip);
            *lock = true;
            TGT_IP_SET.1.notify_one();
        }
    }
    println!();
    match tgt_hostname {
        Some(hostname) => println!(
            "Pinging {} [{}] with {} bytes of data:",
            hostname, tgt_ip, args.size
        ),
        None => println!("Pinging {} with {} bytes of data:", tgt_ip, args.size),
    }

    let icmp_handle = ping::icmp_create()?;
    ping::set_console_handler(Some(console_handler))?;

    let src_addr = match args.srcaddr {
        Some(addr) => addr,
        None => Ipv4Addr::UNSPECIFIED,
    };
    let ttl = match args.ttl {
        Some(ttl) => ttl,
        None => 128,
    };

    let mut done = false;
    while !done {
        let reply = match ping::send_ping(
            icmp_handle,
            src_addr,
            tgt_ip,
            args.size,
            ttl,
            args.dont_fragment,
            args.timeout,
        ) {
            Ok(reply) => Some(reply),
            Err(e) => {
                match e {
                    ping::Error::SendEcho(e) if e.code() == WSA_QOS_ADMISSION_FAILURE.0 as u32 => None,
                    _ => return Err(e.into()),
                }
            }
        };

        let requests_sent = {
            let mut stats = unsafe { STATS.lock().unwrap() };
            match reply {
                Some(reply) => {
                    print_reply_info(&reply);
                    update_stats(&mut stats, &reply)
                }
                None => {
                    stats.requests_sent += 1;
                    println!("Request timed out.");
                }
            }
            stats.requests_sent
        };

        if !args.until_stopped && requests_sent == args.count {
            done = true;
        } else {
            thread::sleep(Duration::from_secs(1));
        }
    }

    let stats = unsafe { STATS.lock().unwrap() };
    print_stats(&stats, tgt_ip);
    Ok(())
}

unsafe extern "system" fn console_handler(ctrl_type: u32) -> BOOL {
    // Wait for the main thread to set the target IP address.
    {
        let mut lock = TGT_IP_SET.0.lock().unwrap();
        while !*lock {
            lock = TGT_IP_SET.1.wait(lock).unwrap();
        }
    }
    let tgt_ip = unsafe { *TGT_IP.lock().unwrap() };
    let tgt_ip = tgt_ip.assume_init();
    let stats = unsafe { STATS.lock().unwrap() };
    print_stats(&stats, tgt_ip);

    if ctrl_type == CTRL_C_EVENT {
        println!("Control-C");
        // Return false so the application is terminated.
        return false.into();
    } else if ctrl_type == CTRL_BREAK_EVENT {
        println!("Control-Break");
        // Return true so the application continues running.
        return true.into();
    }

    false.into()
}

fn get_tgt_ip_and_hostname(args: &CliArgs) -> anyhow::Result<(Ipv4Addr, Option<String>)> {
    let name = &args.target_name;
    match name.parse::<Ipv4Addr>() {
        Ok(ip_addr) => {
            // User specified an IP address.
            let mut hostname: Option<String> = None;
            if args.resolve_addresses {
                // If resolving the IP address to a hostname fails,
                // ignore the error and move on.
                let res = ping::resolve_ip(ip_addr);
                if res.is_ok() {
                    hostname.replace(res.unwrap());
                }
            }
            Ok((ip_addr, hostname))
        }
        Err(_) => {
            // User specified a hostname.
            Ok((ping::resolve_hostname(name)?, Some(name.clone())))
        }
    }
}

fn print_reply_info(reply: &ICMP_ECHO_REPLY) {
    let addr = Ipv4Addr::from(reply.Address.swap_bytes());
    println!(
        "Reply from {}: bytes={} time={}ms TTL={}",
        addr.to_string(),
        reply.DataSize,
        reply.RoundTripTime,
        reply.Options.Ttl
    );
}

fn update_stats(stats: &mut PingStats, reply: &ICMP_ECHO_REPLY) {
    stats.requests_sent += 1;
    stats.replies_rcvd += 1;
    stats.min_rtt = cmp::min(stats.min_rtt, reply.RoundTripTime);
    stats.max_rtt = cmp::max(stats.max_rtt, reply.RoundTripTime);
    let n = stats.requests_sent;
    stats.avg_rtt =
        (((n - 1) * stats.avg_rtt + reply.RoundTripTime) as f64 / n as f64).round() as u32;
}

fn print_stats(stats: &PingStats, tgt_ip: Ipv4Addr) {
    println!();
    println!("Ping statistics for {}:", tgt_ip.to_string());
    let lost = stats.requests_sent - stats.replies_rcvd;
    let loss_perc = (lost as f64 * 100_f64 / stats.requests_sent as f64).round() as u32;
    println!(
        "\tPackets: Sent = {}, Received = {}, Lost = {} ({}% loss),",
        stats.requests_sent, stats.replies_rcvd, lost, loss_perc
    );
    if stats.replies_rcvd > 0 {
        println!("Approximate round trip times in milli-seconds:");
        println!(
            "\tMinimum = {}ms, Maximum = {}ms, Average = {}ms",
            stats.min_rtt, stats.max_rtt, stats.avg_rtt
        );
    }
}

struct PingStats {
    requests_sent: u32,
    replies_rcvd: u32,
    min_rtt: u32,
    max_rtt: u32,
    avg_rtt: u32,
}

impl PingStats {
    const fn new() -> Self {
        PingStats {
            requests_sent: 0,
            replies_rcvd: 0,
            min_rtt: 3600000,
            max_rtt: 0,
            avg_rtt: 0,
        }
    }
}
