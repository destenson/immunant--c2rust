use backtrace::Backtrace;
use log::warn;
use std::any::Any;
use std::cell::Cell;
use std::fmt::Write as _;
use std::panic::{Location, PanicInfo};

#[derive(Clone, Debug)]
pub struct PanicDetail {
    msg: String,
    loc: Option<String>,
    relevant_loc: Option<String>,
    backtrace: Option<Backtrace>,
}

impl PanicDetail {
    pub fn new(msg: String) -> PanicDetail {
        PanicDetail {
            msg,
            loc: None,
            relevant_loc: None,
            backtrace: None,
        }
    }

    pub fn has_backtrace(&self) -> bool {
        self.backtrace.is_some()
    }

    pub fn to_string_short(&self) -> String {
        let loc_str = self.relevant_loc.as_ref().map_or("[unknown]", |s| &*s);
        format!("{}: {}", loc_str, self.msg.trim())
    }

    pub fn to_string_full(&self) -> String {
        let mut s = String::new();
        let loc_str = self.loc.as_ref().map_or("[unknown]", |s| &*s);
        writeln!(s, "panic at {}: {}", loc_str, self.msg).unwrap();
        if let Some(ref bt) = self.backtrace {
            writeln!(s, "{:?}", bt).unwrap();
        }
        s
    }
}

thread_local! {
    static CURRENT_PANIC_DETAIL: Cell<Option<PanicDetail>> = Cell::new(None);
}

pub fn panic_hook(info: &PanicInfo) {
    let bt = Backtrace::new();
    let detail = PanicDetail {
        msg: panic_to_string(info.payload()),
        loc: info.location().map(|l| l.to_string()),
        relevant_loc: guess_relevant_loc(&bt),
        backtrace: Some(bt),
    };
    let old = CURRENT_PANIC_DETAIL.with(|cell| cell.replace(Some(detail)));
    if let Some(old) = old {
        warn!("discarding old panic detail: {:?}", old);
    }
}

pub fn take_current() -> Option<PanicDetail> {
    CURRENT_PANIC_DETAIL.with(|cell| cell.take())
}

pub fn catch(e: &(dyn Any + Send + 'static)) -> PanicDetail {
    take_current().unwrap_or_else(|| {
        let msg = panic_to_string(e);
        warn!("missing panic detail; caught message {:?}", msg);
        PanicDetail::new(msg)
    })
}

fn guess_relevant_loc(bt: &Backtrace) -> Option<String> {
    for frame in bt.frames() {
        for symbol in frame.symbols() {
            let name = match symbol.name() {
                Some(x) => x.to_string(),
                None => continue,
            };
            if name.starts_with("c2rust_analyze::dataflow")
                || name.starts_with("c2rust_analyze::borrowck")
                || name.starts_with("c2rust_analyze::rewrite")
                || name.contains("type_of_rvalue")
                || name.contains("lty_project")
            {
                let filename_str = match symbol.filename() {
                    Some(x) => x.display().to_string(),
                    None => "[unknown]".to_string(),
                };
                return Some(format!(
                    "{} @ {}:{}:{}",
                    name,
                    filename_str,
                    symbol.lineno().unwrap_or(0),
                    symbol.colno().unwrap_or(0)
                ));
            }
        }
    }
    None
}

fn panic_to_string(e: &(dyn Any + Send + 'static)) -> String {
    match e.downcast_ref::<&'static str>() {
        Some(s) => return s.to_string(),
        None => {}
    }

    match e.downcast_ref::<String>() {
        Some(s) => return (*s).clone(),
        None => {}
    }

    format!("unknown error: {:?}", e.type_id())
}