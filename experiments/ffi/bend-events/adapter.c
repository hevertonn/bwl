// Adapter for Bend's internal native effects. Keep this small.
// See bendlang/bend/guide/EFFECTS.md for the current API.
#include <stdint.h>

extern void bwl_events_start(void);
extern uint32_t bwl_events_next(void);
extern void bwl_events_finish(uint32_t);

static Term bwl_events_start_run(Env e, Term *f, IoWork *w) {
  bwl_events_start();
  return term_pak(CID(Unit), 0);
}

static Term bwl_events_next_run(Env e, Term *f, IoWork *w) {
  return (Term)bwl_events_next();
}

static Term bwl_events_finish_run(Env e, Term *f, IoWork *w) {
  bwl_events_finish((uint32_t)f[0]);
  return term_pak(CID(Unit), 0);
}

static void __attribute__((constructor)) bwl_events_register(void) {
  io_eff(CID(Probe.start), bwl_events_start_run);
  io_eff(CID(Probe.next), bwl_events_next_run);
  io_eff(CID(Probe.finish), bwl_events_finish_run);
}
