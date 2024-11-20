use clap::Parser;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};
use std::{fs::File, io::BufReader, path::PathBuf, time::Duration};
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

    /// Only show the raw timestamp ticks on events
    #[clap(long)]
    pub raw_timestamps: bool,

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
    let mut trace_reset_count = 0_u64;
    let mut event_counter_tracker = TrackingEventCounter::zero();
    let mut first_event_observed = false;
    let mut total_dropped_events = 0_u64;
    let mut time_tracker = StreamingInstant::zero();
    let mut context_stats: HashMap<ContextHandle, ContextStats> = Default::default();
    let mut active_context = ContextHandle::Task(ObjectHandle::NO_TASK);
    let mut session_timestamps = Vec::new();

    loop {
        let (event_code, event) = match rd.read_event(&mut r) {
            Ok(Some((ec, ev))) => (ec, ev),
            Ok(None) => break,
            Err(e) => match e {
                Error::TraceRestarted(psf_start_word_endianness) => {
                    warn!("Detected a restarted trace stream");
                    trace_reset_count += 1;
                    first_event_observed = false;
                    active_context = ContextHandle::Task(ObjectHandle::NO_TASK);
                    session_timestamps.push(time_tracker.to_timestamp());
                    rd = RecorderData::read_with_endianness(psf_start_word_endianness, &mut r)?;
                    if let Some(custom_printf_event_id) = opts.custom_printf_event_id {
                        rd.set_custom_printf_event_id(custom_printf_event_id.into());
                    }
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
        let timestamp_dur =
            if !opts.raw_timestamps && !rd.timestamp_info.timer_frequency.is_unitless() {
                let ticks_ns = u128::from(timestamp.get_raw()) * u128::from(ONE_SECOND);
                let total_time_ns =
                    (ticks_ns / u128::from(rd.timestamp_info.timer_frequency.get_raw())) as u64;
                Some(Duration::from_nanos(total_time_ns))
            } else {
                None
            };

        let event_type = event_code.event_type();
        if !opts.no_events && !opts.user_events {
            if let Some(dur) = timestamp_dur {
                print!("[{}.{:03}] ", dur.as_secs(), dur.subsec_millis());
            }
            println!("{event_type} : {event} : {}", event.event_count());
        }
        if opts.user_events {
            if let Event::User(user_event) = &event {
                if let Some(dur) = timestamp_dur {
                    print!("[{}.{:03}] ", dur.as_secs(), dur.subsec_millis());
                }
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
    println!();

    session_timestamps.push(time_tracker.to_timestamp());
    let total_time_ticks: Timestamp = session_timestamps.into_iter().sum();

    let rows: Vec<Vec<Cell>> = rd
        .entry_table
        .entries()
        .iter()
        .map(|(handle, entry)| {
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

            vec![
                Cell::new(handle),
                Cell::new(format!("0x{handle:08X}")),
                Cell::new(entry_class),
                Cell::new(entry_sym),
            ]
        })
        .collect();
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Handle", "Address", "Class", "Symbol"])
        .add_rows(rows);
    for c in table.column_iter_mut() {
        c.set_cell_alignment(CellAlignment::Right);
    }
    table
        .column_mut(3)
        .unwrap()
        .set_cell_alignment(CellAlignment::Left);
    println!("{table}");
    println!();

    let rows: Vec<Vec<Cell>> = observed_type_counters
        .into_iter()
        .sorted_by_key(|t| t.1)
        .map(|(t, count)| {
            let percentage = 100.0 * (count as f64 / total_count as f64);
            vec![
                Cell::new(count),
                Cell::new(format!("{percentage:.01}")),
                Cell::new(format!("0x{:03X}", EventId::from(t))),
                Cell::new(t),
            ]
        })
        .collect();
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Count", "%", "ID", "Type"])
        .add_rows(rows);
    for c in table.column_iter_mut() {
        c.set_cell_alignment(CellAlignment::Right);
    }
    table
        .column_mut(3)
        .unwrap()
        .set_cell_alignment(CellAlignment::Left);
    println!("{table}");
    println!();

    let rows: Vec<Vec<Cell>> = context_stats
        .into_iter()
        .sorted_by_key(|t| t.1.total_runtime.get_raw())
        .map(|(ctx, stats)| {
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
                * ((stats.total_runtime.get_raw() as f64) / (total_time_ticks.get_raw() as f64));
            vec![
                Cell::new(handle),
                Cell::new(sym),
                Cell::new(typ),
                Cell::new(stats.count),
                Cell::new(stats.total_runtime.ticks()),
                Cell::new(total_ns),
                Cell::new(format!("{total_dur:?}")),
                Cell::new(format!("{percentage:.02}")),
            ]
        })
        .collect();
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Handle", "Symbol", "Type", "Count", "Ticks", "Nanos", "Duration", "%",
        ])
        .add_rows(rows);
    for c in table.column_iter_mut() {
        c.set_cell_alignment(CellAlignment::Right);
    }
    table
        .column_mut(1)
        .unwrap()
        .set_cell_alignment(CellAlignment::Left);
    println!("{table}");
    println!();

    println!("----------------------------------------------------------------------------------------------");
    println!("Total events: {total_count}");
    println!("Dropped events: {total_dropped_events}");
    println!("Trace resets: {trace_reset_count}");
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
