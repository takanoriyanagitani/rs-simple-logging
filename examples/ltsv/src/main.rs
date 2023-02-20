use std::collections::BTreeMap;
use std::fmt::Write;
use std::time::{Duration, Instant};

use rs_simple_logging::{
    copy::{
        self, log_debug, log_error, log_fatal, log_info, log_trace, log_warn, logger_new,
        logger_new_from_proxy,
    },
    proxy::copy::{proxy_new_from_resource_proxy, resource_proxy_new_from_map},
    serialize::{serializer_new_from_fn, Serialize},
    write::{level_checker_from_lower_bound, limited_writer_new, log_writer_new_from_fn, LogWrite},
    Item, Severity,
};

fn ltsv_write_ignore_err(buf: &mut String, s: String) {
    write!(buf, "{s}").ok();
}

fn ltsv_serializer() -> impl Serialize {
    serializer_new_from_fn(|i: &Item, buf: &mut String| {
        ltsv_write_ignore_err(buf, format!("level:{}", i.severity.as_str()));
        for pair in &i.attributes {
            let (key, val) = pair;
            ltsv_write_ignore_err(buf, format!("\tattr.{key}:{val}"));
        }
        for pair in &i.resource {
            let (key, val) = pair;
            ltsv_write_ignore_err(buf, format!("\t{key}:{val}"));
        }
        ltsv_write_ignore_err(buf, format!("\tmsg:{}", i.body));
    })
}

fn rate_limiter(min_duration: Duration) -> impl FnMut(Severity) -> bool {
    let mut state = BTreeMap::new();
    move |level: Severity| {
        let prev: Option<Instant> = state.get(&level).copied();
        let duration: Option<Duration> =
            prev.map(|i: Instant| Instant::now().saturating_duration_since(i));
        let available: bool = duration.map(|d: Duration| min_duration < d).unwrap_or(true);
        match available {
            true => {
                state.insert(level, Instant::now());
            }
            false => {}
        }
        available
    }
}

fn ltsv_writer() -> impl LogWrite {
    let writer = log_writer_new_from_fn(
        |serialized: &str, level: Severity| match level {
            Severity::Trace => println!("{serialized}"),
            Severity::Debug => println!("{serialized}"),
            Severity::Info => println!("{serialized}"),
            Severity::Warn => eprintln!("{serialized}"),
            Severity::Error => eprintln!("{serialized}"),
            Severity::Fatal => eprintln!("{serialized}"),
        },
        level_checker_from_lower_bound(Severity::Info),
    );
    limited_writer_new(writer, rate_limiter(Duration::from_millis(1)))
}

fn init_log() {
    let log_writer = ltsv_writer();
    let ser = ltsv_serializer();
    let logger = logger_new(ser, log_writer);

    let resource_proxy = resource_proxy_new_from_map(BTreeMap::from([
        ("service.name".into(), "ltsv-test".into()),
        ("host.ip".into(), "192.168.0.3".into()),
        ("host.name".into(), "instance-a".into()),
    ]));
    let proxy = proxy_new_from_resource_proxy(resource_proxy);

    let logger_with_proxy = logger_new_from_proxy(logger, proxy);

    copy::set_boxed(Box::new(logger_with_proxy));
}

static _RESOURCE_KEYS: &[&str] = &["service.name", "host.ip", "host.name"];

fn trace(msg: &str) {
    log_trace(Item::new(msg, BTreeMap::new()).with_resource_keys(_RESOURCE_KEYS))
}

fn debug(msg: &str) {
    log_debug(Item::new(msg, BTreeMap::new()).with_resource_keys(_RESOURCE_KEYS))
}

fn info(msg: &str) {
    log_info(Item::new(msg, BTreeMap::new()).with_resource_keys(_RESOURCE_KEYS))
}

fn warn(msg: &str) {
    log_warn(Item::new(msg, BTreeMap::new()).with_resource_keys(_RESOURCE_KEYS))
}

fn error(msg: &str) {
    log_error(Item::new(msg, BTreeMap::new()).with_resource_keys(_RESOURCE_KEYS))
}

fn fatal(msg: &str) {
    log_fatal(Item::new(msg, BTreeMap::new()).with_resource_keys(_RESOURCE_KEYS))
}

fn main() {
    init_log();

    trace("Trying to parse request...");
    trace("Trying to parse request...");

    debug("Parameters: id=");
    debug("Parameters: id=");

    info("Path: /users");
    info("Path: /users");

    warn("Invalid request: id missing.");
    warn("Invalid request: id missing.");

    error("Client gone.");
    error("Client gone.");

    fatal("UNABLE TO SAVE LOG");
    fatal("UNABLE TO SAVE LOG");
    std::thread::sleep(Duration::from_millis(10));
    fatal("UNABLE TO SAVE LOG");
}
