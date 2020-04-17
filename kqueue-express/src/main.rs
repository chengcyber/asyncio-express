use std::io::{self, Write};
use std::os::unix::io::AsRawFd;
use std::net::TcpStream;
use std::ptr;

fn main() {
    let mut event_counter = 0;

    let queue = unsafe {
        ffi::kqueue()
    };

    if queue < 0 {
        panic!(io::Error::last_os_error());
    }
let mut streams = vec![];

    for i in 1..6 {
        let addr = "slowwly.robertomurray.co.uk:80";
        let mut stream = TcpStream::connect(addr).unwrap();

        let delay = (5 - i) * 1000;

        let request = format!(
            "GET /delay/{}/url/http://www.google.com HTTP/1.1\r\n
             Host: slowwly.robertomurray.co.uk\r\n
             Connection: close\r\n
             \r\n",
             delay);

        stream.write_all(request.as_bytes()).unwrap();

        stream.set_nonblocking(true).unwrap();

        let event = ffi::Kevent {
            ident: stream.as_raw_fd() as u64,
            filter: ffi::EVFILT_READ,
            flags: ffi::EV_ADD | ffi::EV_ENABLE | ffi::EV_ONESHOT,
            fflags: 0,
            data: 0,
            udata: i,
        };

        let changelist = [event];

        let res = unsafe {
            ffi::kevent(
                queue,
                changelist.as_ptr(),
                1,
                ptr::null_mut(),
                0,
                ptr::null()
                       )
        };

        if res < 0 {
            panic!(io::Error::last_os_error());
        }

        streams.push(stream);
        event_counter += 1;
    }

    while event_counter > 0 {
        let mut events: Vec<ffi::Kevent> = Vec::with_capacity(10);

        let res = unsafe {
            ffi::kevent(
                queue,
                ptr::null(),
                0,
                events.as_mut_ptr(),
                events.capacity() as i32,
                ptr::null(),
                       )
        };

        if res < 0 {
            panic!(io::Error::last_os_error());
        };

        unsafe { events.set_len(res as usize) };

        for event in events {
            println!("RECEIVED: {}", event.udata);
            event_counter -= 1;
        }
    }

    let res = unsafe { ffi::close(queue) };
    if res < 0 {
        panic!(io::Error::last_os_error());
    }

    println!("FINISHIED!");

}

mod ffi {

    pub const EVFILT_READ: i16 = -1;
    pub const EV_ADD: u16 = 0x1;
    pub const EV_ENABLE: u16 = 0x4;
    pub const EV_ONESHOT: u16 = 0x10;

    #[derive(Debug)]
    #[repr(C)]
    pub struct Timespec {
        // Seconds
        tv_sec: isize,
        // Nanoseconds
        v_nsec: usize,
    }

    impl Timespec {
        pub fn from_millis(ms: i32) -> Self {
            let seconds = ms / 1000;
            let nanoseconds = (ms % 1000) * 1000 * 1000;

            Timespec {
                tv_sec: seconds as isize,
                v_nsec: nanoseconds as usize,
            }
        }
    }

    #[derive(Debug, Clone, Default)]
    #[repr(C)]
    pub struct Kevent {
        pub ident: u64,
        pub filter: i16,
        pub flags: u16,
        pub fflags: u32,
        pub data: i64,
        pub udata: u64,
    }

    #[link(name = "c")]
    extern "C" {
        pub(super) fn kqueue() -> i32;

        pub(super) fn kevent(
            kq: i32,
            changelist: *const Kevent,
            nchanges: i32,
            eventlist: *mut Kevent,
            nevents: i32,
            timeout: *const Timespec,
        ) -> i32;

        pub fn close(d: i32) -> i32;
    }
}
