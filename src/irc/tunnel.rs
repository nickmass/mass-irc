use super::tokio::util::channel as tokio;
use super::super::mio::channel as mio;
use std::sync::mpsc::*;
use std::marker::PhantomData;

pub struct ClientTunnel<S, R, O, I> where S: ClientSender<O>, R: ClientReceiver<I> {
    sender: S,
    receiver: R,
    _phantom_out: PhantomData<O>,
    _phantom_in: PhantomData<I>,
    
}

impl<S, R, O, I> ClientTunnel<S, R, O, I>
        where S: ClientSender<O>, R: ClientReceiver<I> {
    pub fn new(sender: S, receiver: R) -> ClientTunnel<S, R, O, I> {
        ClientTunnel {
            sender: sender,
            receiver: receiver,
            _phantom_out: PhantomData,
            _phantom_in: PhantomData,
        }
    }

    pub fn try_read(&self) -> Result<Option<I>, TryRecvError> {
        self.receiver.try_read()
    }

    pub fn write(&self, t: O) -> Result<(), SendError<O>> {
        self.sender.write(t)
    }

    pub fn try_write(&self, t:O) -> Result<(), TrySendError<O>> {
        self.sender.try_write(t)
    }
}

pub trait ClientReceiver<T> {
    fn try_read(&self) -> Result<Option<T>, TryRecvError>;
}

impl<T> ClientReceiver<T> for tokio::Receiver<T> {
    fn try_read(&self) -> Result<Option<T>, TryRecvError> {
        match self.recv() {
            Ok(x) => Ok(x),
            _ => Err(TryRecvError::Disconnected),
        }
    }
}

impl<T> ClientReceiver<T> for Receiver<T> {
    fn try_read(&self) -> Result<Option<T>, TryRecvError> {
        match self.try_recv() {
            Ok(x) => Ok(Some(x)),
            Err(TryRecvError::Empty) => Ok(None),
            _ => Err(TryRecvError::Disconnected),
        }
    }
}

pub trait ClientSender<T> {
    fn write(&self, t: T) -> Result<(), SendError<T>>;

    fn try_write(&self, t:T) -> Result<(), TrySendError<T>>;
}

impl<T> ClientSender<T> for mio::SyncSender<T> {
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

impl<T> ClientSender<T> for SyncSender<T> {
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
