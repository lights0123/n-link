#pragma once
#include "stdint.h"

void * memcpy ( void * destination, const void * source, size_t num );
void * memset ( void * ptr, int value, size_t num );
void * memmove ( void * destination, const void * source, size_t num );
char * strncpy ( char * destination, const char * source, size_t num );
void * memchr ( const void *, int, size_t );
size_t strlen ( const char * str );
void * memset ( void * ptr, int value, size_t num );
