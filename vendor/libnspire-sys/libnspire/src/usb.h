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

#ifndef _USB_H
#define _USB_H

#include "endianconv.h"
#include <libusb.h>

#define NSP_VID 0x0451
#define NSP_PID 0xe012
#define NSP_PID_CX2 0xe022

typedef struct {
	libusb_device_handle *dev;
	unsigned char ep_in, ep_out;
} usb_device_t;

int usb_init();
void usb_finish();
int usb_get_device(usb_device_t *handle, libusb_device_handle *dev);
void usb_free_device(usb_device_t *handle);
int usb_write(usb_device_t *handle, void *ptr, int len);
int usb_read(usb_device_t *handle, void *ptr, int len);

#endif
