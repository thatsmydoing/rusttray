#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate xcb;

mod atom;
mod event;
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

        let mut tray = tray::Tray::new(&conn, &atoms, preferred as usize);
        tray.create();
        if !tray.take_selection() {
            println!("Could not take ownership of tray selection. Maybe another tray is also running?");
            return
        }

        let (tx, rx) = chan::sync::<event::Event>(0);
        {
            let conn = conn.clone();
            thread::spawn(move || {
                event::event_loop(&conn, tx);
            });
        }

        loop {
            use event::Event::*;
            chan_select!(
                rx.recv() -> event => match event.unwrap() {
                    ChildRequest(window) => {
                        tray.adopt(window);
                    },
                    ChildDestroyed(window) => {
                        tray.forget(window);
                    }
                },
                signal.recv() => {
                    break;
                }
            );
        }

        // cleanup code
        tray.cleanup();
    }
    else {
        println!("Could not connect to X server!");
    }
}
