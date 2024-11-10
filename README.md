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
    1     0.0    0x007   DEFINE_ISR
    1     0.0    0x07B   TASK_SUSPEND
    1     0.0    0x014   TIMER_CREATE
 1207     6.6    0x07A   TASK_DELAY
 2400    13.1    0x0EB   UNUSED_STACK
 2887    15.8    0x030   TASK_READY
 4677    25.6    0x037   TASK_ACTIVATE

... runtime stats ...

----------------------------------------------------------------------------------------------
   Handle        Symbol   Type   Count      Ticks         Nanos       Duration      %
----------------------------------------------------------------------------------------------
536876956   LEDTimerISR   ISR       29          0             0            0ns    0.0
536903744       Tmr Svc   Task      60        414        862500        862.5µs    0.0
536896944        Sensor   Task     257       2307       4806250      4.80625ms    0.0
536898216      Actuator   Task     257       3049       6352083     6.352083ms    0.0
536894496       IP-task   Task     523       3882       8087500       8.0875ms    0.0
536912232          EMAC   Task     555       4101       8543750      8.54375ms    0.0
536895784         Comms   Task     264       4399       9164583     9.164583ms    0.0
536900400         Stats   Task      60      22413      46693750     46.69375ms    0.1
536888136        TzCtrl   Task    1200      97286     202679166   202.679166ms    0.6
536901560          IDLE   Task    1501   14850207   30937931250   30.93793125s   99.1
----------------------------------------------------------------------------------------------
Total events: 18303
Dropped events: 0
Total time (ticks): 14988758
Total time: 31226579166ns (31.226579166s)
```

## License

See [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT.
