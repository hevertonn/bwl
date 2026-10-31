import Base

def Probe.start() -> IO(Unit):
  import "./adapter.c"

def Probe.batch() -> IO(List(U32)):
  import "./adapter.c"

def Probe.finish(sum: U32) -> IO(Unit):
  import "./adapter.c"

# Processing happens in Bend, not in the native adapter.
def sum_events(xs: List(U32), acc: U32) -> U32:
  match xs:
    case Nil{}:
      acc
    case Con{head, tail}:
      sum_events(tail, (acc + head : U32))

def consume(fuel: Nat, total: U32) -> IO(U32):
  match fuel:
    case 0n:
      IO.pure(U32, total)
    case 1n+rest:
      do IO<U32>:
        events : List(U32) <- Probe.batch()
        next : U32 = sum_events(events, total)
        consume(rest, next)

def main() -> IO(Unit):
  do IO<Unit>:
    Probe.start()
    checksum : U32 <- consume(1024n, 0)
    Probe.finish(checksum)
