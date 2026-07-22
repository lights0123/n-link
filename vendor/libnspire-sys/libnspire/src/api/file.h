/*
    This file is part of libnspire.

    libnspire is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    libnspire is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with libnspire.  If not, see <http://www.gnu.org/licenses/>.
*/

#ifndef NSP_FILE_H
#define NSP_FILE_H

#include <string.h>
#include "handle.h"

typedef void (*nspire_callback)(size_t, void*);
int nspire_file_write(nspire_handle_t *, const char *, void*, size_t, nspire_callback cb, void *cb_data);
int nspire_file_read(nspire_handle_t *handle, const char *path,
		void* data, size_t size, size_t *read_bytes, nspire_callback cb, void *cb_data);
int nspire_file_move(nspire_handle_t *handle, const char *src, const char *dst);
int nspire_file_copy(nspire_handle_t *handle, const char *src, const char *dst);
int nspire_file_delete(nspire_handle_t *handle, const char *path);

#define nspire_file_touch(h,p)	nspire_file_write((h), (p), NULL, 0);
#define nspire_file_rename	nspire_file_move

#endif
