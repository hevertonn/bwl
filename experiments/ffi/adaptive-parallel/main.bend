import Base

def Probe.start() -> IO(Unit):
  import "./adapter.c"

def Probe.take() -> IO<List<U32>>:
  import "./adapter.c"

def Probe.finish(checksum: U32) -> IO(Unit):
  import "./adapter.c"

# Pure Bend work may be evaluated in parallel by the runtime;
# IO effects remain serialized by the Bend event loop.
def sum(xs: List<U32>, acc: U32) -> U32:
  match xs:
    case Nil{}:
      acc
    case Con{value, tail}:
      sum(tail, (acc + value : U32))

def process(fuel: Nat, checksum: U32) -> IO<U32>:
  match fuel:
    case 0n:
      IO.pure(U32, checksum)
    case 1n+rest:
      do IO<U32>:
        batch : List<U32> <- Probe.take()
        process(rest, sum(batch, checksum))

def main() -> IO(Unit):
  do IO<Unit>:
    Probe.start()
    checksum : U32 <- process(2048n, 0)
    Probe.finish(checksum)
