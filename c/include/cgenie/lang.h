#pragma once
#ifdef __cplusplus
extern "C" {
#endif

typedef void* cgenie_lang_t;

cgenie_lang_t cglang_load_ini(const char* path);
cgenie_lang_t cglang_load_keyval(const char* path);
cgenie_lang_t cglang_load_dll(const char* path);

const char* cglang_get(const cgenie_lang_t file, int index);
const char* cglang_get_named(const cgenie_lang_t file, const char* index);

enum cgenie_lang_error_t {
  CGLANG_SAVE_OK = 0,
  CGLANG_CREATE_ERR = 1,
  CGLANG_WRITE_ERR = 2,
};

enum cgenie_lang_error_t cglang_save_ini(const cgenie_lang_t file, const char* path);
enum cgenie_lang_error_t cglang_save_keyval(const cgenie_lang_t file, const char* path);
enum cgenie_lang_error_t cglang_save_dll(const cgenie_lang_t file, const char* path);

void cglang_free(cgenie_lang_t file);

#ifdef __cplusplus
}
#endif
