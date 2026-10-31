#include <stdint.h>
#include <stddef.h>
struct Pair { uint32_t x; uint32_t y; };
uint32_t fixture_add(uint32_t a, uint32_t b) { return a + b; }
struct Pair fixture_swap(struct Pair p) { return (struct Pair){p.y, p.x}; }
uint64_t fixture_checksum(const uint8_t *p, size_t n) {
  uint64_t sum = 0;
  for (size_t i = 0; i < n; i++) sum += p[i];
  return sum;
}
uint32_t fixture_call(uint32_t (*cb)(uint32_t, void *), uint32_t arg, void *ctx) {
  return cb(arg, ctx);
}
