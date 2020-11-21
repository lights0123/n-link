#pragma once
#include "stdint.h"

void* malloc (size_t size);
void free (void* ptr);
void* realloc (void* ptr, size_t size);
