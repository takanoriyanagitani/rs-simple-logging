//! A simple logging api using non-zero copy.

use std::ops::{Deref, DerefMut};
use std::sync::Mutex;
use std::time::SystemTime;

use crate::{proxy::copy::Proxy, serialize::Serialize, write::LogWrite, Item, Severity};

/// A logger.
pub trait Logger: Sync + Send {
    /// Logs an item.
    fn log(&self, item: Item);
}

/// Creates a logger which converts a log item before logging.
///
/// # Arguments
/// - original: The original logger which will log the converted item.
/// - proxy: Converts a log item.
pub fn logger_new_from_proxy<L, P>(original: L, proxy: P) -> impl Logger
where
    L: Logger,
    P: Proxy + Sync + Send,
{
    logger_new_from_fn(move |old: Item| {
        let neo: Item = proxy.get_item(old);
        original.log(neo)
    })
}

struct FnLogger<L> {
    internal: L,
}

impl<L> Logger for FnLogger<L>
where
    L: Fn(Item) + Sync + Send,
{
    fn log(&self, item: Item) {
        (self.internal)(item)
    }
}

/// Creates a logger from a logger function.
pub fn logger_new_from_fn<L>(internal: L) -> impl Logger
where
    L: Fn(Item) + Sync + Send,
{
    FnLogger { internal }
}

struct WriteSerialized<S, W> {
    serialize: S,
    write: W,
}

impl<S, W> Logger for WriteSerialized<S, W>
where
    S: Serialize,
    W: LogWrite,
{
    fn log(&self, item: Item) {
        let mut buf: String = String::new();
        self.serialize.serialize(&item, &mut buf);
        self.write.write(buf.as_str(), item.severity)
    }
}

/// Creates a logger which writes a serialized log item.
///
/// # Arguments
/// - serialize: Serializes a log item.
/// - write: Writes a serialized log item.
pub fn logger_new<S, W>(serialize: S, write: W) -> impl Logger
where
    S: Serialize,
    W: LogWrite,
{
    WriteSerialized { serialize, write }
}

static _LOGGER: Mutex<Option<&dyn Logger>> = Mutex::new(None);

impl Logger for Option<&dyn Logger> {
    fn log(&self, item: Item) {
        match self {
            None => {}
            Some(l) => l.log(item),
        }
    }
}

impl Logger for Mutex<Option<&dyn Logger>> {
    fn log(&self, item: Item) {
        match self.lock() {
            Err(_) => {}
            Ok(g) => {
                let ro: &Option<_> = g.deref();
                let o: Option<&dyn Logger> = ro.as_deref();
                o.log(item)
            }
        }
    }
}

impl Logger for Box<dyn Logger> {
    fn log(&self, item: Item) {
        let r = self.as_ref();
        r.log(item)
    }
}

fn _log(mut item: Item) {
    item.timestamp = SystemTime::now();
    _LOGGER.log(item)
}

/// Logs an item as a trace-level event.
pub fn log_trace(mut item: Item) {
    item.severity = Severity::Trace;
    _log(item)
}

/// Logs an item as a debugging event.
pub fn log_debug(mut item: Item) {
    item.severity = Severity::Debug;
    _log(item)
}

/// Logs an item as an informational event.
pub fn log_info(mut item: Item) {
    item.severity = Severity::Info;
    _log(item)
}

/// Logs an item as a warning event.
pub fn log_warn(mut item: Item) {
    item.severity = Severity::Warn;
    _log(item)
}

/// Logs an item as an error event.
pub fn log_error(mut item: Item) {
    item.severity = Severity::Error;
    _log(item)
}

/// Logs an item as a fatal event.
pub fn log_fatal(mut item: Item) {
    item.severity = Severity::Fatal;
    _log(item)
}

/// Sets a logger impl.
pub fn set(neo: &'static dyn Logger) {
    match _LOGGER.try_lock() {
        Err(_) => {}
        Ok(mut g) => {
            let mo: &mut Option<_> = g.deref_mut();
            mo.replace(neo);
        }
    }
}

/// Sets a logger impl(boxed).
pub fn set_boxed(neo: Box<dyn Logger>) {
    set(Box::leak(neo))
}
