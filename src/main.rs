use linux_canpil_rs::{CAN, CanModule};

mod frame;
mod socket;

fn main() {
    let can0 = CAN::create(CanModule::CAN0, false).unwrap();
    let can1 = CAN::create(CanModule::CAN1, false).unwrap();
    loop {
        loop {
            let Some(frame) = can0.receive() else { break };
            println!("{frame:?}");
            can0.send(&frame).unwrap();
        }
    }
}
