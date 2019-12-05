#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void* cgenie_scx_t;
typedef enum cgenie_scx_convert_result {
  cgscxOk = 0,
  cgscxScenarioNull = 1,
  cgscxErrCreateFile = 2,
  cgscxErrConvert = 3,
  cgscxErrSerialize = 4,
  cgscxErrUnknownVersion = 5,
} cgenie_scx_result_t;

#define CGSCX_VERSION_AOE "aoe"
#define CGSCX_VERSION_ROR "ror"
#define CGSCX_VERSION_AOC "aoc"
#define CGSCX_VERSION_HD "hd"
#define CGSCX_VERSION_WK "wk"
#define CGSCX_VERSION_KEEP NULL

cgenie_scx_t cgscx_load(const char* input);
cgenie_scx_t cgscx_load_mem(const char* input, size_t size);

cgenie_scx_result_t cgscx_convert_hd_to_wk(cgenie_scx_t scx);
cgenie_scx_result_t cgscx_convert_aoc_to_wk(cgenie_scx_t scx);
cgenie_scx_result_t cgscx_convert_to_wk(cgenie_scx_t scx);

cgenie_scx_result_t cgscx_save(cgenie_scx_t scx, const char* version, const char* output);
/* cgenie_scx_result_t cgscx_save_mem(cgenie_scx_t scx, const char* version, const char** output); */

#ifdef __cplusplus
}
#endif
