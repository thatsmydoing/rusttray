#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate xcb;

mod atom;

use std::thread;
use std::sync::Arc;

fn main() {
    let signal = chan_signal::notify(&[chan_signal::Signal::INT, chan_signal::Signal::TERM]);

    if let Ok((conn, preferred)) = xcb::Connection::connect(None) {
        let conn = Arc::new(conn);
        let atoms = atom::Atoms::new(&conn);

        let setup = conn.get_setup();
        let screen = setup.roots().nth(preferred as usize).unwrap();

        let owner = xcb::get_selection_owner(&conn, atoms.get(&atom::_NET_SYSTEM_TRAY_S0)).get_reply().unwrap().owner();
        if owner != xcb::NONE {
            println!("Another system tray is already running");
            return
        }

        let window = conn.generate_id();
        xcb::create_window(
            &conn,
            xcb::COPY_FROM_PARENT as u8,
            window,
            screen.root(),
            0, 0,
            20, 20,
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            screen.root_visual(),
            &[
                (xcb::CW_BACK_PIXEL, screen.black_pixel()),
                (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE)
            ]
        );
        xcb::map_window(&conn, window);
        conn.flush();

        {
            let conn = conn.clone();
            thread::spawn(move || {
                loop {
                    match conn.wait_for_event() {
                        Some(event) => match event.response_type() {
                            xcb::EXPOSE => { println!("expose") },
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
        xcb::destroy_window(&conn, window);
        conn.flush();
    }
    else {
        println!("Could not connect to X server!");
    }
}
