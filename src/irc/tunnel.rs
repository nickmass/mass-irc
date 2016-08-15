use tokio::util::channel as tokio;
use mio::channel as mio;
use std::sync::mpsc::*;

pub struct ClientTunnel<S, R> where S: ClientSender, R: ClientReceiver {
    sender: S,
    receiver: R,
}

impl<S, R> ClientTunnel<S, R>
        where S: ClientSender, R: ClientReceiver {
    pub fn new(sender: S, receiver: R) -> ClientTunnel<S, R> {
        ClientTunnel {
            sender: sender,
            receiver: receiver,
        }
    }

    pub fn try_read(&self) -> Result<Option<R::Msg>, TryRecvError> {
        self.receiver.try_read()
    }

    pub fn write(&self, t: S::Msg) -> Result<(), SendError<S::Msg>> {
        self.sender.write(t)
    }

    pub fn try_write(&self, t: S::Msg) -> Result<(), TrySendError<S::Msg>> {
        self.sender.try_write(t)
    }
}

pub trait ClientReceiver {
    type Msg;
    fn try_read(&self) -> Result<Option<Self::Msg>, TryRecvError>;
}

impl<T> ClientReceiver for tokio::Receiver<T> {
    type Msg = T;
    fn try_read(&self) -> Result<Option<T>, TryRecvError> {
        match self.recv() {
            Ok(x) => Ok(x),
            _ => Err(TryRecvError::Disconnected),
        }
    }
}

impl<T> ClientReceiver for Receiver<T> {
    type Msg = T;
    fn try_read(&self) -> Result<Option<T>, TryRecvError> {
        match self.try_recv() {
            Ok(x) => Ok(Some(x)),
            Err(TryRecvError::Empty) => Ok(None),
            _ => Err(TryRecvError::Disconnected),
        }
    }
}

pub trait ClientSender {
    type Msg;
    fn write(&self, t: Self::Msg) -> Result<(), SendError<Self::Msg>>;

    fn try_write(&self, t:Self::Msg) -> Result<(), TrySendError<Self::Msg>>;
}

impl<T> ClientSender for mio::SyncSender<T> {
    type Msg = T;
    fn write(&self, t: T) -> Result<(), SendError<T>> {
        match self.send(t) {
            Ok(_) => Ok(()),
            Err(mio::SendError::Disconnected(i)) => Err(SendError(i)),
            _ => unimplemented!(),
        }
    }

    fn try_write(&self, t:T) -> Result<(), TrySendError<T>> {
        match self.try_send(t) {
            Ok(_) => Ok(()),
            Err(mio::TrySendError::Full(i))=> Err(TrySendError::Full(i)),
            Err(mio::TrySendError::Disconnected(i))=> Err(TrySendError::Disconnected(i)),
            _ => unimplemented!(),
        }
    }
}

impl<T> ClientSender for SyncSender<T> {
    type Msg = T;
    fn write(&self, t: T) -> Result<(), SendError<T>> {
        match self.send(t) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn try_write(&self, t:T) -> Result<(), TrySendError<T>> {
        match self.try_send(t) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
