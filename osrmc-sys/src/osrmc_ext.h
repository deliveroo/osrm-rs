#ifndef OSRMC_EXT_H_
#define OSRMC_EXT_H_

/*
 * Additions to libosrmc for raw results and snap cache counters. They need an
 * OSRM build that provides OSRM::TableRaw, OSRM::TableRows, OSRM::RouteRaw and
 * OSRM::GetSnapCacheCounters; stock OSRM 5.26 does not have them (see
 * osrmc_ext_raw_available).
 * Errors follow libosrmc: on failure *error is set and owned by the caller.
 */

#include "libosrmc/libosrmc/osrmc.h"

#ifdef __cplusplus
extern "C" {
#endif

/* 1 if the functions below were built against an OSRM that provides them. Built against stock OSRM,
   osrmc_table_raw, osrmc_table_rows and osrmc_route_raw fail with NotImplemented and the snap cache
   counters are zero. */
OSRMC_API int osrmc_ext_raw_available(void);

/* Map the dataset files (on, the default) or read them into process memory (off). */
OSRMC_API void osrmc_config_set_use_mmap(osrmc_config_t config, int on);

/* A table as the search computed it: durations in deciseconds (INT32_MAX = no route) and distances in
   metres (FLT_MAX = no route), row-major sources x destinations. No response object is built. */
typedef struct osrmc_table_raw_result* osrmc_table_raw_t;
OSRMC_API osrmc_table_raw_t osrmc_table_raw(osrmc_osrm_t osrm, osrmc_table_params_t params, osrmc_error_t* error);
OSRMC_API void osrmc_table_raw_destruct(osrmc_table_raw_t raw);
OSRMC_API const int* osrmc_table_raw_durations(osrmc_table_raw_t raw, unsigned long* len);
OSRMC_API const float* osrmc_table_raw_distances(osrmc_table_raw_t raw, unsigned long* len);

/* Like osrmc_table_raw, but each finished source row is passed to `on_row` instead of being stored. `on_row` is
   called once per row, possibly from several threads at the same time; the pointers are only valid during the
   call. `distances` is NULL if the parameters do not ask for distances. */
typedef void (*osrmc_table_row_fn)(void* context, unsigned long row, const int* durations, const float* distances);
OSRMC_API void osrmc_table_rows(osrmc_osrm_t osrm, osrmc_table_params_t params, osrmc_table_row_fn on_row,
                                void* context, osrmc_error_t* error);

/* Duration (seconds) and distance (metres) of the first route, the values osrmc_route_response_duration and
   osrmc_route_response_distance report, without building a response. Not available with steps. */
OSRMC_API void osrmc_route_raw(osrmc_osrm_t osrm, osrmc_route_params_t params, float* duration, float* distance,
                               osrmc_error_t* error);

/* Counters of the engine's snap cache (OSRM::GetSnapCacheCounters), all zero when it is off. */
typedef struct osrmc_snap_cache_counters {
  unsigned long long hits;
  unsigned long long misses;
  unsigned long long admissions;
  unsigned long long evictions;
  unsigned long long entries;
  unsigned long long capacity;
} osrmc_snap_cache_counters_t;
OSRMC_API void osrmc_snap_cache_counters(osrmc_osrm_t osrm, osrmc_snap_cache_counters_t* counters);

#ifdef __cplusplus
}
#endif

#endif
