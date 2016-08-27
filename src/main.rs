#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate getopts;
extern crate xcb;

mod atom;
mod tray;

use std::env;
use std::process;
use std::thread;
use std::sync::Arc;

const PROGRAM: &'static str = "rustray";
const EXIT_WRONG_ARGS: i32 = 1;
const EXIT_FAILED_CONNECT: i32 = 10;
const EXIT_FAILED_SELECT: i32 = 11;

const CLIENT_MESSAGE: u8 = xcb::CLIENT_MESSAGE | 0x80;

fn main() {
    process::exit(real_main());
}

fn real_main() -> i32 {
    let signal = chan_signal::notify(&[chan_signal::Signal::INT, chan_signal::Signal::TERM]);
    let args: Vec<String> = env::args().collect();

    let mut opts = getopts::Options::new();
    opts.optopt("i", "icon-size", "size of the tray icons, default 20", "<size>");
    opts.optopt("p", "position", "position of the tray, one of: top-left, top-right, bottom-left, bottom-right", "<pos>");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string())
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options]", PROGRAM);
        print!("{}", opts.usage(&brief));
        return 0
    }
    let pos = matches.opt_str("p").unwrap_or("top-left".to_string());
    let pos = match pos.as_ref() {
        "top-left" => tray::TOP_LEFT,
        "top-right" => tray::TOP_RIGHT,
        "bottom-left" => tray::BOTTOM_LEFT,
        "bottom-right" => tray::BOTTOM_RIGHT,
        _ => {
            println!("Invalid position specified.");
            return EXIT_WRONG_ARGS
        }
    };
    let size = matches.opt_str("i");
    let size = match size {
        Some(string) => match string.parse::<u16>() {
            Ok(size) => size,
            Err(e) => {
                println!("Invalid size specified, {}.", e.to_string());
                return EXIT_WRONG_ARGS
            }
        },
        None => 20
    };

    if let Ok((conn, preferred)) = xcb::Connection::connect(None) {
        let conn = Arc::new(conn);
        let atoms = atom::Atoms::new(&conn);

        let mut tray = tray::Tray::new(&conn, &atoms, preferred as usize, size, pos);

        if !tray.is_selection_available() {
            println!("Another system tray is already running");
            return EXIT_FAILED_SELECT
        }

        let (tx, rx) = chan::sync(0);
        {
            let conn = conn.clone();
            thread::spawn(move || {
                loop {
                    match conn.wait_for_event() {
                        Some(event) => { tx.send(event); },
                        None => { break; }
                    }
                }
            });
        }

        tray.create();

        let mut ready = false;
        loop {
            chan_select!(
                rx.recv() -> event => {
                    let event = event.unwrap();
                    match event.response_type() {
                        xcb::PROPERTY_NOTIFY if !ready => {
                            ready = true;
                            let event: &xcb::PropertyNotifyEvent = xcb::cast_event(&event);
                            if !tray.take_selection(event.time()) {
                                println!("Could not take ownership of tray selection. Maybe another tray is also running?");
                                return EXIT_FAILED_SELECT
                            }
                        },
                        CLIENT_MESSAGE => {
                            let event: &xcb::ClientMessageEvent = xcb::cast_event(&event);
                            let data = event.data().data32();
                            let window = data[2];
                            tray.adopt(window);
                        },
                        xcb::DESTROY_NOTIFY => {
                            let event: &xcb::DestroyNotifyEvent = xcb::cast_event(&event);
                            tray.forget(event.window());
                        },
                        xcb::CONFIGURE_NOTIFY => {
                            let event: &xcb::ConfigureNotifyEvent = xcb::cast_event(&event);
                            tray.force_size(event.window(), Some((event.width(), event.height())));
                        },
                        _ => {}
                    }
                },
                signal.recv() => {
                    break;
                }
            );
        }

        // cleanup code
        tray.cleanup();
        return 0
    }
    else {
        println!("Could not connect to X server!");
        return EXIT_FAILED_CONNECT
    }
}
