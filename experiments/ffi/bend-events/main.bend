import Base

# The producer runs on a separate Rust thread.
# Each IO effect returns one real U32 value into Bend.
def Probe.start() -> IO(Unit):
  import "./adapter.c"

def Probe.next() -> IO(U32):
  import "./adapter.c"

def Probe.finish(sum: U32) -> IO(Unit):
  import "./adapter.c"

# The accumulator is computed in Bend, not Rust or the adapter.
# The fuel also guarantees termination in the proof-checked language.
def consume(fuel: Nat, sum: U32) -> IO(U32):
  match fuel:
    case 0n:
      IO.pure(U32, sum)
    case 1n+rest:
      do IO<U32>:
        event : U32 <- Probe.next()
        consume(rest, (sum + event : U32))

def main() -> IO(Unit):
  do IO<Unit>:
    Probe.start()
    checksum : U32 <- consume(20000n, 0)
    Probe.finish(checksum)
