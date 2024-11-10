# trace-recorder-printer

Print Percepio TraceRecorder streaming data from file

## Install

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
  -h, --help                                             Print help
  -V, --version                                          Print version
```

## Examples

```text
trace-recorder-printer /tmp/test_system.psf

RecorderData {
    protocol: Streaming,
    header: HeaderInfo {
        endianness: Little,
        format_version: 14,
        kernel_version: KernelVersion(
            [
                161,
                26,
            ],
        ),
        kernel_port: FreeRtos,
        options: 0,
        irq_priority_order: 0,
        num_cores: 1,
        isr_tail_chaining_threshold: 0,
        platform_cfg: "FreeRTOS",
        platform_cfg_version: PlatformCfgVersion {
            major: 1,
            minor: 2,
            patch: 0,
        },
    },
    timestamp_info: TimestampInfo {
        timer_type: FreeRunning32Incr,
        timer_frequency: Frequency(
            480000,
        ),
        timer_period: 0,
        timer_wraparounds: 0,
        os_tick_rate_hz: Frequency(
            1000,
        ),
        latest_timestamp: Timestamp(
            0,
        ),
        os_tick_count: 0,
    },
... events ...
TRACE_START : TraceStart([150]:(startup)) : 6
OBJECT_NAME : ObjectName([150]:536877100:'info') : 7
OBJECT_NAME : ObjectName([150]:536877052:'warn') : 8
OBJECT_NAME : ObjectName([150]:536877004:'error') : 9
USER_EVENT : User([150]:[info]='Initializing LED ISR') : 10
... symbol table ...
--------------------------------------------------------
   Handle           Class    Symbol
--------------------------------------------------------
        2            Task    (startup)
536874508                    System Heap
536875372                    Actuator
536875420                    Comms
536875468                    Sensor
... event stats ...
--------------------------------------------------------
Count       %    ID      Type
--------------------------------------------------------
    1     0.0    0x001   TRACE_START
   34     0.2    0x003   OBJECT_NAME
    1     0.0    0x007   DEFINE_ISR
    8     0.0    0x010   TASK_CREATE
 2887    15.8    0x030   TASK_READY
   29     0.2    0x033   TASK_SWITCH_ISR_BEGIN
... runtime stats ...
----------------------------------------------------------------------------------------------
   Handle        Symbol   Type   Count      Ticks         Nanos       Duration      %
----------------------------------------------------------------------------------------------
536900400         Stats   Task      60      22413      46693750     46.69375ms    0.1
536876956   LEDTimerISR   ISR       29          0             0            0ns    0.0
536894496       IP-task   Task     523       3882       8087500       8.0875ms    0.0
536888136        TzCtrl   Task    1200      97286     202679166   202.679166ms    0.6
536898216      Actuator   Task     257       3049       6352083     6.352083ms    0.0
536896944        Sensor   Task     257       2307       4806250      4.80625ms    0.0
536903744       Tmr Svc   Task      60        414        862500        862.5µs    0.0
536895784         Comms   Task     264       4399       9164583     9.164583ms    0.0
536912232          EMAC   Task     555       4101       8543750      8.54375ms    0.0
536901560          IDLE   Task    1501   14850207   30937931250   30.93793125s   99.1
----------------------------------------------------------------------------------------------
Total events: 18303
Dropped events: 0
Total time (ticks): 14988758
Total time: 31226579166ns (31.226579166s)
```

## License

See [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT.
