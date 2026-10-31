import Base

def Probe.ping() -> IO(Unit):
  import "./ping.c"

def main() -> IO(Unit):
  do IO<Unit>:
    Probe.ping()
    IO.print("BEND_AFTER_RUST")
