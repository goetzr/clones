use std::ffi::c_void;
use std::fmt;
use std::mem::{self, MaybeUninit};
use std::net::{Ipv4Addr, SocketAddrV4};

use windows::core::PCWSTR;
use windows::Win32::Foundation::*;
use windows::Win32::NetworkManagement::Dns::*;
use windows::Win32::NetworkManagement::IpHelper::*;
use windows::Win32::Networking::WinSock::*;
use windows::Win32::System::Console::*;
use windows::Win32::System::WindowsProgramming::*;

#[derive(Debug)]
pub enum Error {
    ConsoleHandler(wp::Error),
    InitWinsock(wp::Error),
    ResolveHostname(wp::Error),
    ResolveIpAddr(wp::Error),
    IcmpHandle(wp::Error),
    SendEcho(wp::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            ConsoleHandler(e) => write!(f, "failed to set console handler: {}", e),
            InitWinsock(e) => write!(f, "failed to initialize winsock: {}", e),
            ResolveHostname(e) => write!(f, "failed to resolve hostname to IP address: {}", e),
            ResolveIpAddr(e) => write!(f, "failed to resolve IP address to hostname: {}", e),
            IcmpHandle(e) => write!(f, "failed to open an ICMP handle: {}", e),
            SendEcho(e) => write!(f, "failed to send the echo request: {}", e),
        }
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

pub fn set_console_handler(handler: PHANDLER_ROUTINE) -> Result<()> {
    if unsafe { SetConsoleCtrlHandler(handler, true) }.as_bool() {
        Ok(())
    } else {
        Err(Error::ConsoleHandler(wp::last_error()))
    }
}

pub fn init_winsock() -> Result<()> {
    let version: u16 = 2 << 8 | 2;
    let mut data = MaybeUninit::<WSADATA>::uninit();
    if unsafe { WSAStartup(version, data.as_mut_ptr()) } == 0 {
        Ok(())
    } else {
        Err(Error::InitWinsock(wp::last_error()))
    }
}

pub fn resolve_hostname(hostname: &str) -> Result<Ipv4Addr> {
    let hostname_utf16 = wp::utf8_to_utf16(hostname);
    let mut query_results = MaybeUninit::<&DNS_RECORDA>::uninit();
    unsafe {
        DnsQuery_W(
            PCWSTR::from_raw(hostname_utf16.as_ptr()),
            DNS_TYPE_A,
            DNS_QUERY_STANDARD,
            None,
            Some(query_results.as_mut_ptr() as *mut *mut DNS_RECORDA),
            None,
        )
        .ok()
        .map_err(|e| Error::ResolveHostname(wp::Error::from_win_error(e)))?;

        let query_results = query_results.assume_init();
        let ip_addr = Ipv4Addr::from(query_results.Data.A.IpAddress.swap_bytes());

        DnsFree(
            Some(query_results as *const DNS_RECORDA as *const c_void),
            DnsFreeRecordList,
        );

        Ok(ip_addr)
    }
}

pub fn resolve_ip(ip_addr: Ipv4Addr) -> Result<String> {
    let sock_addr = SOCKADDR_IN::from(SocketAddrV4::new(ip_addr, 0));
    let mut hostname: [MaybeUninit<u16>; NI_MAXHOST as usize] =
        unsafe { MaybeUninit::uninit().assume_init() };
    if unsafe {
        GetNameInfoW(
            &sock_addr as *const SOCKADDR_IN as *const SOCKADDR,
            mem::size_of::<SOCKADDR_IN>() as i32,
            Some(&mut *(&mut hostname as *mut [MaybeUninit<u16>] as *mut [u16])),
            None,
            0,
        )
    } == 0
    {
        let hostname = &hostname as *const [MaybeUninit<u16>] as *const [u16] as *const u16;
        Ok(wp::utf16_to_utf8(hostname))
    } else {
        Err(Error::ResolveIpAddr(wp::last_error()))
    }
}

pub fn icmp_create() -> Result<IcmpHandle> {
    unsafe { IcmpCreateFile().map_err(|e| Error::IcmpHandle(wp::Error::from_win_error(e))) }
}

fn build_request_data(size: u16) -> Vec<u8> {
    (0..size)
        .into_iter()
        .map(|n| (b'A' as u16 + n % 26) as u8)
        .collect()
}

fn get_request_options(ttl: u8, dont_fragment: bool) -> IP_OPTION_INFORMATION {
    IP_OPTION_INFORMATION {
        Ttl: ttl,
        Tos: 0,
        Flags: if dont_fragment { IP_FLAG_DF as u8 } else { 0 },
        OptionsSize: 0,
        OptionsData: std::ptr::null::<u8>() as *mut u8,
    }
}

fn build_reply_buffer(sz_request_data: usize) -> Vec<MaybeUninit<u8>> {
    let mut buf: Vec<MaybeUninit<u8>> = Vec::new();
    let sz_reply_buf =
        mem::size_of::<ICMP_ECHO_REPLY>() + sz_request_data + 8 + mem::size_of::<IO_STATUS_BLOCK>();
    buf.reserve(sz_reply_buf);
    buf
}

pub fn send_ping(
    icmp_handle: IcmpHandle,
    src_addr: Ipv4Addr,
    dst_addr: Ipv4Addr,
    size: u16,
    ttl: u8,
    dont_fragment: bool,
    timeout: u32,
) -> Result<ICMP_ECHO_REPLY> {
    let request_data = build_request_data(size);
    let request_options = get_request_options(ttl, dont_fragment);
    let mut reply_buf = build_reply_buffer(request_data.len());

    let num_replies = unsafe {
        IcmpSendEcho2Ex(
            icmp_handle,
            HANDLE(0), // Event
            None,      // ApcRoutine
            None,      // ApcContext
            Into::<u32>::into(src_addr).swap_bytes(),
            Into::<u32>::into(dst_addr).swap_bytes(),
            request_data.as_ptr() as *const c_void,
            request_data.len() as u16,
            Some(&request_options as *const IP_OPTION_INFORMATION),
            reply_buf.as_mut_ptr() as *mut c_void,
            reply_buf.capacity() as u32,
            timeout,
        )
    };
    if num_replies == 0 {
        Err(Error::SendEcho(wp::last_error()))
    } else {
        Ok(unsafe { *(reply_buf.as_ptr() as *const ICMP_ECHO_REPLY) })
    }
}
