#pragma once

#include <stdint.h>

#define RUST_PROBE_MAGIC 0x52555354u

uint32_t rust_kernel_probe(void);
