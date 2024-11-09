use clap::Parser;
use std::collections::BTreeMap;
use std::{fs::File, io::BufReader, path::PathBuf, time::Duration};
use tabular::{Row, Table};
use trace_recorder_parser::{
    streaming::{
        event::{Event, EventId, TrackingEventCounter},
        Error, RecorderData,
    },
    time::StreamingInstant,
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
        println!("{rd:#?}");
    }

    if opts.no_events {
        return Ok(());
    }

    let mut observed_type_counters = BTreeMap::new();
    let mut total_count = 0_u64;
    let mut event_counter_tracker = TrackingEventCounter::zero();
    let mut first_event_observed = false;
    let mut total_dropped_events = 0_u64;
    let mut time_tracker = StreamingInstant::zero();

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

        let _abs_timestamp = time_tracker.elapsed(event.timestamp());

        let event_type = event_code.event_type();
        if !opts.user_events {
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
    }

    if opts.user_events {
        return Ok(());
    }

    let mut table = Table::new("{:>}    {:>}    {:<}");
    table.add_heading("--------------------------------------------------------");
    table.add_row(
        Row::new()
            .with_cell("Handle")
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
    for (t, count) in observed_type_counters.into_iter() {
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

    let total_time_ticks = time_tracker.to_timestamp();
    println!("--------------------------------------------------------");
    println!("Total events: {total_count}");
    println!("Dropped events: {total_dropped_events}");
    println!("Total time (ticks): {}", total_time_ticks);
    if !rd.timestamp_info.timer_frequency.is_unitless() {
        const ONE_SECOND: u64 = 1_000_000_000;
        let ticks_ns = u128::from(total_time_ticks.get_raw()) * u128::from(ONE_SECOND);
        let total_time_ns =
            (ticks_ns / u128::from(rd.timestamp_info.timer_frequency.get_raw())) as u64;
        let total_dur = Duration::from_nanos(total_time_ns);
        println!("Total time: {}ns ({:?})", total_time_ns, total_dur);
    }

    Ok(())
}

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
