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

#ifndef NSP_HANDLE_H
#define NSP_HANDLE_H

#include <stdbool.h>

typedef struct nspire_handle nspire_handle_t;
typedef struct libusb_device_handle libusb_device_handle;

int nspire_init(nspire_handle_t **ptr, libusb_device_handle *dev, bool is_cx2);
void nspire_free(nspire_handle_t *ptr);

#endif
