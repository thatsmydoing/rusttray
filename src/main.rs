#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate xcb;

mod atom;
mod tray;

use std::thread;
use std::sync::Arc;

fn main() {
    let signal = chan_signal::notify(&[chan_signal::Signal::INT, chan_signal::Signal::TERM]);

    if let Ok((conn, preferred)) = xcb::Connection::connect(None) {
        let conn = Arc::new(conn);
        let atoms = atom::Atoms::new(&conn);

        let owner = xcb::get_selection_owner(&conn, atoms.get(atom::_NET_SYSTEM_TRAY_S0)).get_reply().unwrap().owner();
        if owner != xcb::NONE {
            println!("Another system tray is already running");
            return
        }

        let tray = tray::Tray::new(&conn, &atoms, preferred as usize);
        tray.create();
        if !tray.take_selection() {
            println!("Could not take ownership of tray selection. Maybe another tray is also running?");
            return
        }

        {
            let conn = conn.clone();
            const CLIENT_MESSAGE: u8 = xcb::CLIENT_MESSAGE | 0x80;
            thread::spawn(move || {
                loop {
                    match conn.wait_for_event() {
                        Some(event) => match event.response_type() {
                            xcb::EXPOSE => { println!("expose") },
                            CLIENT_MESSAGE => {
                                println!("client message");
                            },
                            _ => {}
                        },
                        None => { break; }
                    }
                }
            });
        }

        loop {
            chan_select!(
                signal.recv() => {
                    break;
                }
            );
        }

        // cleanup code
    }
    else {
        println!("Could not connect to X server!");
    }
}
