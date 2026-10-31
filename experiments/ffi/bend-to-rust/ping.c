// Bend 2 native effect adapter. Relies on the compiler's current internal API.
// See bend2/effs/print.c; this is not a promised stable ABI.
extern void bwl_rust_probe_ping(void);

Term probe_ping_run(Env e, Term* f, IoWork* w) {
  bwl_rust_probe_ping();
  return term_pak(CID(Unit), 0);
}
static void __attribute__((constructor)) probe_ping_use(void) {
  io_eff(CID(Probe.ping), probe_ping_run);
}
