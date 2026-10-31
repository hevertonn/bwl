// Small compiler-specific adapter. All queue/benchmark logic is Rust.
// Its symbols and io_eff API belong to the experimental Bend runtime.
extern void bwl_bench_start(void);
extern void bwl_bench_consume(void);
extern void bwl_bench_finish(void);

static Term bwl_start_run(Env e, Term *f, IoWork *w) {
  bwl_bench_start();
  return term_pak(CID(Unit), 0);
}

static Term bwl_consume_run(Env e, Term *f, IoWork *w) {
  bwl_bench_consume();
  return term_pak(CID(Unit), 0);
}

static Term bwl_finish_run(Env e, Term *f, IoWork *w) {
  bwl_bench_finish();
  return term_pak(CID(Unit), 0);
}

static void __attribute__((constructor)) bwl_effects_register(void) {
  io_eff(CID(Probe.start), bwl_start_run);
  io_eff(CID(Probe.consume), bwl_consume_run);
  io_eff(CID(Probe.finish), bwl_finish_run);
}
