#include <cstddef>
#include <cstdint>
#include <memory>
#include <stdexcept>
#include <string>
#include <vector>

#include <osrm/engine_config.hpp>
#include <osrm/json_container.hpp>
#include <osrm/osrm.hpp>
#include <osrm/route_parameters.hpp>
#include <osrm/status.hpp>
#include <osrm/table_parameters.hpp>

#include "osrmc_ext.h"

/* Same definition as in libosrmc/osrmc.cc, so that osrmc_error_code/_message/_destruct apply. */
struct osrmc_error final {
  std::string code;
  std::string message;
};

struct osrmc_table_raw_result final {
  std::vector<std::int32_t> durations;
  std::vector<float> distances;
};

void osrmc_config_set_use_mmap(osrmc_config_t config, int on) {
  reinterpret_cast<osrm::EngineConfig*>(config)->use_mmap = on != 0;
}

void osrmc_table_raw_destruct(osrmc_table_raw_t raw) { delete raw; }

const int* osrmc_table_raw_durations(osrmc_table_raw_t raw, unsigned long* len) {
  *len = raw->durations.size();
  return raw->durations.data();
}

const float* osrmc_table_raw_distances(osrmc_table_raw_t raw, unsigned long* len) {
  *len = raw->distances.size();
  return raw->distances.data();
}

#ifndef OSRMC_EXT_STOCK_OSRM

namespace {
void error_from_exception(const std::exception& e, osrmc_error_t* error) {
  *error = new osrmc_error{"Exception", e.what()};
}

void error_from_json(osrm::json::Object& json, osrmc_error_t* error) try {
  auto code = json.values["code"].get<osrm::json::String>().value;
  auto message = json.values["message"].get<osrm::json::String>().value;
  if (code.empty()) {
    code = "Unknown";
  }
  *error = new osrmc_error{code, message};
} catch (const std::exception& e) {
  error_from_exception(e, error);
}
}  // namespace

int osrmc_ext_raw_available(void) { return 1; }

osrmc_table_raw_t osrmc_table_raw(osrmc_osrm_t osrm, osrmc_table_params_t params, osrmc_error_t* error) try {
  auto* osrm_typed = reinterpret_cast<osrm::OSRM*>(osrm);
  auto* params_typed = reinterpret_cast<osrm::TableParameters*>(params);

  auto out = std::make_unique<osrmc_table_raw_result>();
  osrm::json::Object failure;
  const auto status = osrm_typed->TableRaw(*params_typed, out->durations, out->distances, failure);

  if (status == osrm::Status::Ok)
    return out.release();

  error_from_json(failure, error);
  return nullptr;
} catch (const std::exception& e) {
  error_from_exception(e, error);
  return nullptr;
}

void osrmc_table_rows(osrmc_osrm_t osrm, osrmc_table_params_t params, osrmc_table_row_fn on_row, void* context,
                      osrmc_error_t* error) try {
  auto* osrm_typed = reinterpret_cast<osrm::OSRM*>(osrm);
  auto* params_typed = reinterpret_cast<osrm::TableParameters*>(params);

  osrm::json::Object failure;
  const auto status = osrm_typed->TableRows(
      *params_typed,
      [on_row, context](std::size_t row, const std::int32_t* durations, const float* distances) {
        on_row(context, row, durations, distances);
      },
      failure);

  if (status != osrm::Status::Ok)
    error_from_json(failure, error);
} catch (const std::exception& e) {
  error_from_exception(e, error);
}

void osrmc_route_raw(osrmc_osrm_t osrm, osrmc_route_params_t params, float* duration, float* distance,
                     osrmc_error_t* error) try {
  auto* osrm_typed = reinterpret_cast<osrm::OSRM*>(osrm);
  auto* params_typed = reinterpret_cast<osrm::RouteParameters*>(params);

  double raw_duration = 0.;
  double raw_distance = 0.;
  osrm::json::Object failure;
  const auto status = osrm_typed->RouteRaw(*params_typed, raw_duration, raw_distance, failure);

  if (status != osrm::Status::Ok) {
    error_from_json(failure, error);
    return;
  }
  // Same narrowing as osrmc_route_response_duration / _distance.
  *duration = raw_duration;
  *distance = raw_distance;
} catch (const std::exception& e) {
  error_from_exception(e, error);
}

void osrmc_snap_cache_counters(osrmc_osrm_t osrm, osrmc_snap_cache_counters_t* counters) {
  const auto value = reinterpret_cast<osrm::OSRM*>(osrm)->GetSnapCacheCounters();
  *counters = osrmc_snap_cache_counters_t{value.hits,      value.misses,  value.admissions,
                                          value.evictions, value.entries, value.capacity};
}

#else

namespace {
void not_implemented(osrmc_error_t* error) {
  *error = new osrmc_error{"NotImplemented", "needs an OSRM build with TableRaw, TableRows and RouteRaw"};
}
}  // namespace

int osrmc_ext_raw_available(void) { return 0; }

osrmc_table_raw_t osrmc_table_raw(osrmc_osrm_t, osrmc_table_params_t, osrmc_error_t* error) {
  not_implemented(error);
  return nullptr;
}

void osrmc_table_rows(osrmc_osrm_t, osrmc_table_params_t, osrmc_table_row_fn, void*, osrmc_error_t* error) {
  not_implemented(error);
}

void osrmc_route_raw(osrmc_osrm_t, osrmc_route_params_t, float*, float*, osrmc_error_t* error) { not_implemented(error); }

void osrmc_snap_cache_counters(osrmc_osrm_t, osrmc_snap_cache_counters_t* counters) {
  *counters = osrmc_snap_cache_counters_t{0, 0, 0, 0, 0, 0};
}

#endif
