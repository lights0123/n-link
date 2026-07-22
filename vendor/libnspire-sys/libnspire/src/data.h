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

#ifndef _DATA_H
#define _DATA_H

#include <stdint.h>
#include "handle.h"
#include "endianconv.h"
#include "packet.h"

int data_write_special(nspire_handle_t*, void*, size_t,
		void (*)(struct packet *));

int data_write(nspire_handle_t *handle, void *ptr, size_t maxlen);
int data_read(nspire_handle_t *handle, void *ptr, size_t maxlen, size_t *actual);
int data_build(const char *, void *, size_t, size_t *, ...);
int data_scan(const char *, const void *, size_t, ...);

static inline int data_write8(nspire_handle_t *handle, uint8_t x) {
	return data_write(handle, &x, 1);
}

static inline int data_write16(nspire_handle_t *handle, uint16_t x) {
	x = dcpu16(x);
	return data_write(handle, &x, 2);
}

#endif
