// Compiler-specific glue: Rust fills a native buffer; the effect constructs
// an immutable Bend List(U32) from those values. No event is reduced in C.
#include <stdint.h>
#include <stdlib.h>

extern void bwl_batches_start(void);
extern uint32_t bwl_batches_take(uint32_t *out, uint32_t capacity);
extern void bwl_batches_finish(uint32_t checksum);

static Term start_run(Env e, Term *f, IoWork *w) {
  bwl_batches_start();
  return term_pak(CID(Unit), 0);
}

static Term batch_run(Env e, Term *f, IoWork *w) {
  uint32_t values[256];
  uint32_t count = bwl_batches_take(values, 256);

  Term list = term_pak(CID(Nil), 0);
  for (uint32_t i = count; i > 0; --i) {
    list = io_node(e, CID(Con), (Term)values[i - 1], list);
  }
  return list;
}

static Term finish_run(Env e, Term *f, IoWork *w) {
  bwl_batches_finish((uint32_t)f[0]);
  return term_pak(CID(Unit), 0);
}

static void __attribute__((constructor)) register_bwl_batches(void) {
  io_eff(CID(Probe.start), start_run);
  io_eff(CID(Probe.batch), batch_run);
  io_eff(CID(Probe.finish), finish_run);
}
