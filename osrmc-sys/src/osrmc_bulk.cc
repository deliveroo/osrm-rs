#include <cmath>
#include <exception>
#include <string>

#include <osrm/json_container.hpp>

#include "osrmc_bulk.h"

/*
 * Must stay token-for-token identical to the definition in libosrmc/libosrmc/osrmc.cc (the struct
 * is opaque in osrmc.h). Identical definitions across translation units are permitted by the ODR,
 * which is what lets errors allocated here be freed by libosrmc's osrmc_error_destruct.
 */
struct osrmc_error final {
  std::string code;
  std::string message;
};

namespace {

void set_error(osrmc_error_t* error, const char* code, const char* message) noexcept try {
  *error = new osrmc_error{code, message};
} catch (...) {
  /* Allocation failed: nothing more we can report. Leave *error untouched. */
}

size_t fill_matrix(osrmc_table_response_t response, const char* key, const char* missing_message, float* out,
                   size_t len, osrmc_error_t* error) noexcept try {
  const auto* response_typed = reinterpret_cast<const osrm::json::Object*>(response);

  const auto table = response_typed->values.find(key);
  if (table == response_typed->values.end()) {
    set_error(error, "NoTable", missing_message);
    return 0;
  }

  const auto& rows = table->second.get<osrm::json::Array>().values;

  size_t cells = 0;
  for (const auto& row : rows) {
    cells += row.get<osrm::json::Array>().values.size();
  }
  if (cells > len) {
    const auto message = "Output buffer holds " + std::to_string(len) + " cells, table has " + std::to_string(cells);
    set_error(error, "BufferTooSmall", message.c_str());
    return 0;
  }

  size_t written = 0;
  for (const auto& row : rows) {
    for (const auto& cell : row.get<osrm::json::Array>().values) {
      /* Same semantics as osrmc_table_response_duration/distance: null means unreachable. */
      out[written++] =
          cell.is<osrm::json::Null>() ? INFINITY : static_cast<float>(cell.get<osrm::json::Number>().value);
    }
  }
  return written;
} catch (const std::exception& e) {
  set_error(error, "Exception", e.what());
  return 0;
} catch (...) {
  set_error(error, "Exception", "Unknown exception");
  return 0;
}

}  // namespace

size_t osrmc_table_response_durations(osrmc_table_response_t response, float* out, size_t len,
                                      osrmc_error_t* error) {
  return fill_matrix(response, "durations", "Table request not configured to return durations", out, len, error);
}

size_t osrmc_table_response_distances(osrmc_table_response_t response, float* out, size_t len,
                                      osrmc_error_t* error) {
  return fill_matrix(response, "distances", "Table request not configured to return distances", out, len, error);
}
