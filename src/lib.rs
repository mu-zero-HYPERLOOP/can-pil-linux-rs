use std::sync::mpsc::Receiver;

use frame::CanError;
use socket::OwnedCanSocket;

use crate::frame::CanFrame;

mod frame;
mod socket;

pub enum CanModule {
    CAN0,
    CAN1,
}

pub struct CAN {
    socket: OwnedCanSocket,
    rx: Receiver<CanFrame>,
    err_rx: Receiver<CanError>,
}

impl CAN {
    pub fn create(module: CanModule, recv_errors: bool) -> Result<CAN, std::io::Error> {
        let ifname = match module {
            CanModule::CAN0 => "can0",
            CanModule::CAN1 => "can1",
        };
        let socket = OwnedCanSocket::open(ifname)?;

        let (tx, rx) = std::sync::mpsc::channel::<CanFrame>();
        let (err_tx, err_rx) = std::sync::mpsc::channel::<CanError>();

        let socket_ref = socket.as_ref();
        std::thread::spawn(move || loop {
            match socket_ref.receive() {
                Ok(frame) => {
                    if tx.send(frame).is_err() {
                        break;
                    }
                }
                Err(err) => {
                    if recv_errors && err_tx.send(err).is_err() {
                        break;
                    }
                }
            }
        });

        Ok(CAN { socket, rx, err_rx })
    }

    pub fn send(&self, frame: &CanFrame) -> Result<(), CanError> {
        self.socket.transmit(frame)
    }

    pub fn receive(&self) -> Option<CanFrame> {
        match self.rx.try_recv() {
            Ok(frame) => Some(frame),
            Err(err) => match err {
                std::sync::mpsc::TryRecvError::Empty => None,
                std::sync::mpsc::TryRecvError::Disconnected => panic!("rx transmitter dropped"),
            },
        }
    }

    pub fn receive_blocking(&self) -> Option<CanFrame> {
        match self.rx.recv() {
            Ok(frame) => Some(frame),
            Err(_) => panic!("rx transmitter dropped"),
        }
    }
    pub fn receive_err(&self) -> Option<CanError> {
        match self.err_rx.try_recv() {
            Ok(err) => Some(err),
            Err(err) => match err {
                std::sync::mpsc::TryRecvError::Empty => None,
                std::sync::mpsc::TryRecvError::Disconnected => panic!("err_rx transmitter dropped"),
            },
        }
    }
}
