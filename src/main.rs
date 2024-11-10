use clap::Parser;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};
use std::{fs::File, io::BufReader, path::PathBuf, time::Duration};
use tabular::{Row, Table};
use trace_recorder_parser::{
    streaming::{
        event::{Event, EventId, IsrEvent, TaskEvent, TrackingEventCounter},
        Error, RecorderData,
    },
    time::{StreamingInstant, Timestamp},
    types::ObjectHandle,
};
use tracing::{error, warn};

#[derive(Parser, Debug, Clone)]
#[clap(version, about = "Print Percepio TraceRecorder streaming data from file", long_about = None)]
pub struct Opts {
    /// Don't print events
    #[clap(long)]
    pub no_events: bool,

    /// Custom printf event ID
    #[clap(long, value_parser=clap_num::maybe_hex::<u16>)]
    pub custom_printf_event_id: Option<u16>,

    /// Only print user event formatted strings
    #[clap(long, conflicts_with = "no_events")]
    pub user_events: bool,

    /// Path to streaming data file (psf)
    #[clap(value_parser)]
    pub path: PathBuf,
}

fn main() {
    match do_main() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{e}");
            let mut cause = e.source();
            while let Some(err) = cause {
                eprintln!("Caused by: {err}");
                cause = err.source();
            }
            std::process::exit(exitcode::SOFTWARE);
        }
    }
}

fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();

    reset_signal_pipe_handler()?;

    tracing_subscriber::fmt::init();

    let f = File::open(&opts.path)?;
    let mut r = BufReader::new(f);

    let mut rd = RecorderData::find(&mut r)?;

    if let Some(custom_printf_event_id) = opts.custom_printf_event_id {
        rd.set_custom_printf_event_id(custom_printf_event_id.into());
    }

    if !opts.user_events {
        println!("Protocol: {}", rd.protocol);
        println!("Header");
        println!("  - Endianness: {}", rd.header.endianness);
        println!("  - Format version: {}", rd.header.format_version);
        println!("  - Kernel version: {}", rd.header.kernel_version);
        println!("  - Kernel port: {}", rd.header.kernel_port);
        println!("  - Options: 0x{:X}", rd.header.options);
        println!("  - IRQ priority order: {}", rd.header.irq_priority_order);
        println!("  - Cores: {}", rd.header.num_cores);
        println!(
            "  - ISR tail chaining threshold: {}",
            rd.header.isr_tail_chaining_threshold
        );
        println!("  - Platform config: {}", rd.header.platform_cfg);
        println!(
            "  - Platform config version: {}",
            rd.header.platform_cfg_version
        );
        println!("Timestamp Info");
        println!("  - Timer type: {}", rd.timestamp_info.timer_type);
        println!("  - Timer frequency: {}", rd.timestamp_info.timer_frequency);
        println!("  - Timer period: {}", rd.timestamp_info.timer_period);
        println!(
            "  - Timer wraparounds: {}",
            rd.timestamp_info.timer_wraparounds
        );
        println!("  - OS tick rate Hz: {}", rd.timestamp_info.os_tick_rate_hz);
        println!(
            "  - Latest timestamp: {}",
            rd.timestamp_info.latest_timestamp
        );
        println!("  - OS tick count: {}", rd.timestamp_info.os_tick_count);
    }

    let mut observed_type_counters = BTreeMap::new();
    let mut total_count = 0_u64;
    let mut event_counter_tracker = TrackingEventCounter::zero();
    let mut first_event_observed = false;
    let mut total_dropped_events = 0_u64;
    let mut time_tracker = StreamingInstant::zero();
    let mut context_stats: HashMap<ContextHandle, ContextStats> = Default::default();
    let mut active_context = ContextHandle::Task(ObjectHandle::NO_TASK);

    loop {
        let (event_code, event) = match rd.read_event(&mut r) {
            Ok(Some((ec, ev))) => (ec, ev),
            Ok(None) => break,
            Err(e) => match e {
                Error::TraceRestarted(psf_start_word_endianness) => {
                    warn!("Detected a restarted trace stream");
                    first_event_observed = false;
                    rd = RecorderData::read_with_endianness(psf_start_word_endianness, &mut r)?;
                    if let Some(custom_printf_event_id) = opts.custom_printf_event_id {
                        rd.set_custom_printf_event_id(custom_printf_event_id.into());
                    }
                    // TODO - probably can do better and maintain these stats
                    context_stats.clear();
                    continue;
                }
                _ => {
                    error!("{e}");
                    continue;
                }
            },
        };

        let dropped_events = if !first_event_observed {
            event_counter_tracker.set_initial_count(event.event_count());
            time_tracker = StreamingInstant::new(
                event.timestamp().ticks() as u32,
                rd.timestamp_info.timer_wraparounds,
            );
            first_event_observed = true;
            None
        } else {
            event_counter_tracker.update(event.event_count())
        };

        let timestamp = time_tracker.elapsed(event.timestamp());

        let event_type = event_code.event_type();
        if !opts.no_events {
            println!("{event_type} : {event} : {}", event.event_count());
        }
        if opts.user_events {
            if let Event::User(user_event) = &event {
                println!("{user_event}");
            }
        }

        *observed_type_counters.entry(event_type).or_insert(0) += 1_u64;
        total_count += 1;

        if let Some(dropped_events) = dropped_events {
            warn!(
                event_count = u16::from(event.event_count()),
                dropped_events, "Dropped events detected"
            );
            total_dropped_events += dropped_events;
        }

        // Update active context and stats
        let maybe_contex_switch_handle: Option<ContextHandle> = match &event {
            Event::IsrBegin(ev) | Event::IsrResume(ev) => Some(ev.into()),
            Event::TaskBegin(ev) | Event::TaskResume(ev) | Event::TaskActivate(ev) => {
                Some(ev.into())
            }
            _ => None,
        };

        if let Some(contex_switch_handle) = maybe_contex_switch_handle {
            if contex_switch_handle != active_context {
                // Update runtime stats for the previous context being switched out
                if let Some(prev_ctx_stats) = context_stats.get_mut(&active_context) {
                    prev_ctx_stats.update(timestamp);
                }

                // Same for the new context being switched in
                let ctx_stats = context_stats
                    .entry(contex_switch_handle)
                    .or_insert_with(|| ContextStats::new(timestamp));
                ctx_stats.set_last_timestamp(timestamp);

                active_context = contex_switch_handle;
            }
        }
    }

    if opts.user_events {
        return Ok(());
    }

    let mut table = Table::new("{:>}    {:>}    {:>}    {:<}");
    table.add_heading("--------------------------------------------------------");
    table.add_row(
        Row::new()
            .with_cell("Handle")
            .with_cell("Address")
            .with_cell("Class")
            .with_cell("Symbol"),
    );
    table.add_heading("--------------------------------------------------------");
    for (handle, entry) in rd.entry_table.entries().iter() {
        let entry_class = if let Some(c) = entry.class {
            c.to_string()
        } else {
            "".to_owned()
        };
        let entry_sym = if let Some(s) = &entry.symbol {
            s.as_ref()
        } else {
            ""
        };

        table.add_row(
            Row::new()
                .with_cell(handle)
                .with_cell(format!("0x{handle:08X}"))
                .with_cell(entry_class)
                .with_cell(entry_sym),
        );
    }
    print!("{table}");

    let mut table = Table::new("{:>}    {:>}    {:<}   {:<}");
    table.add_heading("--------------------------------------------------------");
    table.add_row(
        Row::new()
            .with_cell("Count")
            .with_cell("%")
            .with_cell("ID")
            .with_cell("Type"),
    );
    table.add_heading("--------------------------------------------------------");
    for (t, count) in observed_type_counters.into_iter().sorted_by_key(|t| t.1) {
        let percentage = 100.0 * (count as f64 / total_count as f64);
        table.add_row(
            Row::new()
                .with_cell(count)
                .with_cell(format!("{percentage:.01}"))
                .with_cell(format!("0x{:03X}", EventId::from(t)))
                .with_cell(t),
        );
    }
    print!("{table}");

    let mut table = Table::new("{:>}   {:>}   {:<}   {:>}   {:>}   {:>}   {:>}   {:>}");
    table.add_heading(
        "----------------------------------------------------------------------------------------------",
    );
    table.add_row(
        Row::new()
            .with_cell("Handle")
            .with_cell("Symbol")
            .with_cell("Type")
            .with_cell("Count")
            .with_cell("Ticks")
            .with_cell("Nanos")
            .with_cell("Duration")
            .with_cell("%"),
    );
    table.add_heading(
        "----------------------------------------------------------------------------------------------",
    );
    for (ctx, stats) in context_stats
        .into_iter()
        .sorted_by_key(|t| t.1.total_runtime.get_raw())
    {
        let handle = ctx.object_handle();
        let sym = rd
            .entry_table
            .symbol(handle)
            .map(|s| s.as_ref())
            .unwrap_or("");
        let typ = match ctx {
            ContextHandle::Task(_) => "Task",
            ContextHandle::Isr(_) => "ISR",
        };
        let total_ns = if !rd.timestamp_info.timer_frequency.is_unitless() {
            let ticks_ns = u128::from(stats.total_runtime.get_raw()) * u128::from(ONE_SECOND);
            (ticks_ns / u128::from(rd.timestamp_info.timer_frequency.get_raw())) as u64
        } else {
            0
        };
        let total_dur = Duration::from_nanos(total_ns);
        let percentage = 100.0
            * ((stats.total_runtime.get_raw() as f64)
                / (time_tracker.to_timestamp().get_raw() as f64));
        table.add_row(
            Row::new()
                .with_cell(handle)
                .with_cell(sym)
                .with_cell(typ)
                .with_cell(stats.count)
                .with_cell(stats.total_runtime.ticks())
                .with_cell(total_ns)
                .with_cell(format!("{total_dur:?}"))
                .with_cell(format!("{percentage:.01}")),
        );
    }
    print!("{table}");

    let total_time_ticks = time_tracker.to_timestamp();
    println!("----------------------------------------------------------------------------------------------");
    println!("Total events: {total_count}");
    println!("Dropped events: {total_dropped_events}");
    println!("Total time (ticks): {}", total_time_ticks);
    if !rd.timestamp_info.timer_frequency.is_unitless() {
        let ticks_ns = u128::from(total_time_ticks.get_raw()) * u128::from(ONE_SECOND);
        let total_time_ns =
            (ticks_ns / u128::from(rd.timestamp_info.timer_frequency.get_raw())) as u64;
        let total_dur = Duration::from_nanos(total_time_ns);
        println!("Total time (ns): {}", total_time_ns);
        println!("Total time: {:?}", total_dur);
    }

    Ok(())
}

// ns
const ONE_SECOND: u64 = 1_000_000_000;

// Used to prevent panics on broken pipes.
// See:
//   https://github.com/rust-lang/rust/issues/46016#issuecomment-605624865
fn reset_signal_pipe_handler() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_family = "unix")]
    {
        use nix::sys::signal;

        unsafe {
            signal::signal(signal::Signal::SIGPIPE, signal::SigHandler::SigDfl)?;
        }
    }

    Ok(())
}

type DurationTicks = Timestamp;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct ContextStats {
    /// When the context was last switched in
    last_timestamp: Timestamp,

    /// Total time the context has been in the running state
    total_runtime: DurationTicks,

    /// Number of times the context was switched in
    count: u64,
}

impl ContextStats {
    fn new(last_timestamp: Timestamp) -> Self {
        Self {
            last_timestamp,
            total_runtime: DurationTicks::zero(),
            count: 0,
        }
    }

    /// Called when this context is switched in
    fn set_last_timestamp(&mut self, last_timestamp: Timestamp) {
        self.last_timestamp = last_timestamp;
        self.count += 1;
    }

    /// Called when this context is switched out
    fn update(&mut self, timestamp: Timestamp) {
        if timestamp < self.last_timestamp {
            warn!("Stats timestamp went backwards");
        } else {
            let diff = timestamp - self.last_timestamp;
            self.total_runtime += diff;
            self.last_timestamp = timestamp;
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum ContextHandle {
    Task(ObjectHandle),
    Isr(ObjectHandle),
}

impl ContextHandle {
    fn object_handle(self) -> ObjectHandle {
        match self {
            ContextHandle::Task(h) => h,
            ContextHandle::Isr(h) => h,
        }
    }
}

impl From<&TaskEvent> for ContextHandle {
    fn from(event: &TaskEvent) -> Self {
        ContextHandle::Task(event.handle)
    }
}

impl From<&IsrEvent> for ContextHandle {
    fn from(event: &IsrEvent) -> Self {
        ContextHandle::Isr(event.handle)
    }
}
