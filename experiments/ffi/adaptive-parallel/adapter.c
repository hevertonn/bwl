#include <stdint.h>
extern void bwl_adaptive_start(void);
extern uint32_t bwl_adaptive_take(uint32_t *out, uint32_t cap);
extern void bwl_adaptive_finish(uint32_t checksum);

static Term start_run(Env e, Term *f, IoWork *w) {
  bwl_adaptive_start();
  return term_pak(CID(Unit), 0);
}

static Term take_run(Env e, Term *f, IoWork *w) {
  uint32_t data[256];
  uint32_t n = bwl_adaptive_take(data, 256);
  Term xs = term_pak(CID(Nil), 0);
  for (uint32_t i = n; i > 0; --i) {
    xs = io_node(e, CID(Con), (Term)data[i - 1], xs);
  }
  return xs;
}

static Term finish_run(Env e, Term *f, IoWork *w) {
  bwl_adaptive_finish((uint32_t)f[0]);
  return term_pak(CID(Unit), 0);
}

static void __attribute__((constructor)) register_adaptive(void) {
  io_eff(CID(Probe.start), start_run);
  io_eff(CID(Probe.take), take_run);
  io_eff(CID(Probe.finish), finish_run);
}
