#ifndef OSRMC_BULK_H_
#define OSRMC_BULK_H_

/*
 * Bulk extraction of table responses, layered on top of libosrmc without modifying it.
 *
 * libosrmc only exposes per-cell accessors (osrmc_table_response_duration/distance), each doing
 * two hash lookups plus a JSON value copy per cell, and heap-allocating an osrmc_error for every
 * unreachable cell. These functions walk the response once instead.
 */

#include <stddef.h>

#include "libosrmc/libosrmc/osrmc.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Write the full durations matrix of `response` into `out` in row-major order
 * (`out[source * num_destinations + destination]`). Unreachable cells (JSON null) are written as
 * INFINITY.
 *
 * Returns the number of cells written. On failure returns 0 and sets `*error` (caller takes
 * ownership, free with osrmc_error_destruct):
 *   - "NoTable"        the response contains no "durations" key
 *   - "BufferTooSmall" `len` is smaller than the number of cells in the matrix; nothing is written
 *   - "Exception"      the response had an unexpected shape, or any other C++ exception
 */
OSRMC_API size_t osrmc_table_response_durations(osrmc_table_response_t response, float* out, size_t len,
                                                osrmc_error_t* error);

/* As osrmc_table_response_durations, for the "distances" matrix. */
OSRMC_API size_t osrmc_table_response_distances(osrmc_table_response_t response, float* out, size_t len,
                                                osrmc_error_t* error);

#ifdef __cplusplus
}
#endif

#endif
