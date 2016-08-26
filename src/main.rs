#[macro_use]
extern crate chan;
extern crate chan_signal;

fn main() {
    let signal = chan_signal::notify(&[chan_signal::Signal::INT, chan_signal::Signal::TERM]);

    println!("Hello, world!");

    loop {
        chan_select!(
            signal.recv() => {
                break;
            }
        );
    }

    println!("Cleaning up...");
}
