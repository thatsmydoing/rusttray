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

        let dir = "top-right";
        let dir = match dir {
            "top-right" => tray::TOP_RIGHT,
            "bottom-left" => tray::BOTTOM_LEFT,
            "bottom-right" => tray::BOTTOM_RIGHT,
            _ => tray::TOP_LEFT
        };

        let mut tray = tray::Tray::new(&conn, &atoms, preferred as usize, 20, dir);

        if !tray.is_selection_available() {
            println!("Another system tray is already running");
            return
        }

        let (tx, rx) = chan::sync::<event::Event>(0);
        {
            let conn = conn.clone();
            thread::spawn(move || {
                event::event_loop(&conn, tx);
            });
        }

        tray.create();

        loop {
            use event::Event::*;
            chan_select!(
                rx.recv() -> event => match event.unwrap() {
                    Ready(timestamp) => {
                        if !tray.take_selection(timestamp) {
                            println!("Could not take ownership of tray selection. Maybe another tray is also running?");
                            return
                        }
                    },
                    ChildRequest(window) => {
                        tray.adopt(window);
                    },
                    ChildDestroyed(window) => {
                        tray.forget(window);
                    },
                    ChildConfigured(window) => {
                        tray.force_size(window);
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
