use std::ops::DerefMut;
use std::sync::Mutex;

use crate::Severity;

pub trait LogWrite: Sync + Send {
    fn write(&self, serialized: &str, level: Severity);
}

struct LimitedWrite<L, S> {
    writer: L,
    state: Mutex<S>,
}

impl<L, S> LogWrite for LimitedWrite<L, S>
where
    L: LogWrite,
    S: FnMut(Severity) -> bool + Sync + Send,
{
    fn write(&self, serialized: &str, level: Severity) {
        match self.state.lock() {
            Err(_) => {}
            Ok(mut guard) => {
                let state: &mut S = guard.deref_mut();
                let log_available: bool = state(level);
                match log_available {
                    true => self.writer.write(serialized, level),
                    false => {}
                }
            }
        }
    }
}

/// Creates a log writer which can ignore a log item.
///
/// # Arguments
/// - original: The original log writer.
/// - log_available: Checks if a log item for a severity can be accepted or not.
pub fn limited_writer_new<L, S>(original: L, log_available: S) -> impl LogWrite
where
    L: LogWrite,
    S: FnMut(Severity) -> bool + Sync + Send,
{
    LimitedWrite {
        writer: original,
        state: Mutex::new(log_available),
    }
}

struct FnWrite<W, L> {
    internal: W,
    check_level: L,
}

impl<W, L> LogWrite for FnWrite<W, L>
where
    W: Fn(&str, Severity) + Sync + Send,
    L: Fn(Severity) -> bool + Sync + Send,
{
    fn write(&self, serialized: &str, level: Severity) {
        match (self.check_level)(level) {
            false => {}
            true => (self.internal)(serialized, level),
        }
    }
}

pub fn log_writer_new_from_fn<W, L>(internal: W, check_level: L) -> impl LogWrite
where
    W: Fn(&str, Severity) + Sync + Send,
    L: Fn(Severity) -> bool + Sync + Send,
{
    FnWrite {
        internal,
        check_level,
    }
}

pub fn log_writer_new_std_default_from_fn<L>(check_level: L) -> impl LogWrite
where
    L: Fn(Severity) -> bool + Sync + Send,
{
    log_writer_new_from_fn(
        |serialized: &str, level: Severity| match level {
            Severity::Trace => println!("{serialized}"),
            Severity::Debug => println!("{serialized}"),
            Severity::Info => println!("{serialized}"),
            Severity::Warn => eprintln!("{serialized}"),
            Severity::Error => eprintln!("{serialized}"),
            Severity::Fatal => eprintln!("{serialized}"),
        },
        check_level,
    )
}

pub fn level_checker_from_lower_bound(
    lb_inclusive: Severity,
) -> impl Fn(Severity) -> bool + Send + Sync {
    let lbi: u8 = lb_inclusive.into();
    move |level: Severity| {
        let u: u8 = level.into();
        lbi <= u
    }
}

pub fn log_writer_new_std_default_from_lower_bound(lb_inclusive: Severity) -> impl LogWrite {
    log_writer_new_std_default_from_fn(level_checker_from_lower_bound(lb_inclusive))
}
