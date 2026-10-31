import Base

def Probe.start() -> IO(Unit):
  import "./adapter.c"

def Probe.consume() -> IO(Unit):
  import "./adapter.c"

def Probe.finish() -> IO(Unit):
  import "./adapter.c"

# Each recursion step executes a genuine Bend IO effect. Rust never
# calls a Bend function. The native producer runs on another OS thread.
def consume_batches(fuel: Nat) -> IO(Unit):
  match fuel:
    case 0:
      IO.pure(Unit, Unit{})
    case 1 n:
      do IO<Unit>:
        Probe.consume()
        consume_batches(n)

def main() -> IO(Unit):
  do IO<Unit>:
    Probe.start()
    consume_batches(1024)
    Probe.finish()
