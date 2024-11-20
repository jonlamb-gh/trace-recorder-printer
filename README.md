# trace-recorder-printer

Print Percepio TraceRecorder streaming data from file

## Install

* Download one of the pre-built [releases](https://github.com/jonlamb-gh/trace-recorder-printer/releases)
* Or build/install from source:
  ```bash
  git clone https://github.com/jonlamb-gh/trace-recorder-printer.git
  cd trace-recorder-printer
  cargo install --path .
  ```

## CLI

```text
trace-recorder-printer --help
Print Percepio TraceRecorder streaming data from file

Usage: trace-recorder-printer [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to streaming data file (psf)

Options:
      --no-events                                        Don't print events
      --custom-printf-event-id <CUSTOM_PRINTF_EVENT_ID>  Custom printf event ID
      --user-events                                      Only print user event formatted strings
      --raw-timestamps                                   Only show the raw timestamp ticks on events
  -h, --help                                             Print help
  -V, --version                                          Print version
```

## Examples

Note that the output (tables/events/etc) have been truncated for clarity.

```text
trace-recorder-printer /tmp/test_system.psf

Protocol: streaming
Header
  - Endianness: little-endian
  - Format version: 14
  - Kernel version: [A1, 1A]
  - Kernel port: FreeRTOS
  - Options: 0x0
  - IRQ priority order: 0
  - Cores: 1
  - ISR tail chaining threshold: 0
  - Platform config: FreeRTOS
  - Platform config version: 1.2.0
Timestamp Info
  - Timer type: FreeRunning32Incr
  - Timer frequency: 480000
  - Timer period: 0
  - Timer wraparounds: 0
  - OS tick rate Hz: 1000
  - Latest timestamp: 0
  - OS tick count: 0

... events ...

[0.000] TRACE_START : TraceStart([117]:(startup)) : 6
[0.000] OBJECT_NAME : ObjectName([119]:536877072:'info') : 7
[0.000] OBJECT_NAME : ObjectName([121]:536877024:'warn') : 8
[0.000] OBJECT_NAME : ObjectName([124]:536876976:'error') : 9
[0.000] USER_EVENT : User([127]:[info]='Initializing LED ISR') : 10
[0.000] DEFINE_ISR : IsrDefine([130]:'LEDTimerISR':4) : 11

... symbol table ...

╭───────────┬────────────┬──────────────┬─────────────╮
│    Handle ┆    Address ┆        Class ┆ Symbol      │
╞═══════════╪════════════╪══════════════╪═════════════╡
│         2 ┆ 0x00000002 ┆         Task ┆ (startup)   │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 536874480 ┆ 0x20000DF0 ┆              ┆ System Heap │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 536875344 ┆ 0x20001150 ┆              ┆ Actuator    │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 536875392 ┆ 0x20001180 ┆              ┆ Comms       │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 536903512 ┆ 0x20007F58 ┆    Semaphore ┆             │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 536911880 ┆ 0x2000A008 ┆         Task ┆ EMAC        │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 536912088 ┆ 0x2000A0D8 ┆   EventGroup ┆             │
╰───────────┴────────────┴──────────────┴─────────────╯

... event stats ...

╭───────┬──────┬───────┬───────────────────────────╮
│ Count ┆    % ┆    ID ┆ Type                      │
╞═══════╪══════╪═══════╪═══════════════════════════╡
│     1 ┆  0.0 ┆ 0x001 ┆ TRACE_START               │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│     1 ┆  0.0 ┆ 0x007 ┆ DEFINE_ISR                │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│     1 ┆  0.0 ┆ 0x07B ┆ TASK_SUSPEND              │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│     1 ┆  0.0 ┆ 0x014 ┆ TIMER_CREATE              │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│     1 ┆  0.0 ┆ 0x0A0 ┆ TIMER_START               │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│     1 ┆  0.0 ┆ 0x015 ┆ EVENTGROUP_CREATE         │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│     1 ┆  0.0 ┆ 0x0B1 ┆ EVENTGROUP_WAITBITS       │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│     1 ┆  0.0 ┆ 0x0B4 ┆ EVENTGROUP_SETBITS        │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│  2887 ┆ 15.8 ┆ 0x030 ┆ TASK_READY                │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│  4677 ┆ 25.6 ┆ 0x037 ┆ TASK_ACTIVATE             │
╰───────┴──────┴───────┴───────────────────────────╯

... runtime stats ...

╭───────────┬─────────────┬──────┬───────┬──────────┬─────────────┬───────────────┬───────╮
│    Handle ┆ Symbol      ┆ Type ┆ Count ┆    Ticks ┆       Nanos ┆      Duration ┆     % │
╞═══════════╪═════════════╪══════╪═══════╪══════════╪═════════════╪═══════════════╪═══════╡
│ 536876928 ┆ LEDTimerISR ┆  ISR ┆    29 ┆       94 ┆      195833 ┆     195.833µs ┆  0.00 │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ 536903392 ┆ Tmr Svc     ┆ Task ┆    60 ┆      275 ┆      572916 ┆     572.916µs ┆  0.00 │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ 536896592 ┆ Sensor      ┆ Task ┆   257 ┆     2153 ┆     4485416 ┆    4.485416ms ┆  0.02 │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ 536897864 ┆ Actuator    ┆ Task ┆   257 ┆     2762 ┆     5754166 ┆    5.754166ms ┆  0.02 │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ 536894144 ┆ IP-task     ┆ Task ┆   523 ┆     3593 ┆     7485416 ┆    7.485416ms ┆  0.03 │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ 536911880 ┆ EMAC        ┆ Task ┆   555 ┆     4013 ┆     8360416 ┆    8.360416ms ┆  0.03 │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ 536895432 ┆ Comms       ┆ Task ┆   264 ┆     4255 ┆     8864583 ┆    8.864583ms ┆  0.03 │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ 536900048 ┆ Stats       ┆ Task ┆    60 ┆    21302 ┆    44379166 ┆   44.379166ms ┆  0.15 │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ 536887784 ┆ TzCtrl      ┆ Task ┆  1200 ┆    92407 ┆   192514583 ┆  192.514583ms ┆  0.65 │
├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌┤
│ 536901208 ┆ IDLE        ┆ Task ┆  1501 ┆ 14143288 ┆ 29465183333 ┆ 29.465183333s ┆ 99.08 │
╰───────────┴─────────────┴──────┴───────┴──────────┴─────────────┴───────────────┴───────╯

----------------------------------------------------------------------------------------------
Total events: 18303
Dropped events: 0
Trace resets: 0
Total time (ticks): 14274788
Total time (ns): 29739141666
Total time: 29.739141666s
```

## License

See [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT.
