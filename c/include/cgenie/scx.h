#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void* cgenie_scx_t;

cgenie_scx_t cgscx_load(const char* input);
cgenie_scx_t cgscx_load_mem(const char* input, size_t size);

uint32_t cgscx_convert_hd_to_wk(cgenie_scx_t scx);

uint32_t cgscx_save(cgenie_scx_t scx, const char* output);
uint32_t cgscx_save_mem(cgenie_scx_t scx, const char** output);

#ifdef __cplusplus
}
#endif
