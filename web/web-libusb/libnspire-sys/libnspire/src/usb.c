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

#include "usb.h"
#include "error.h"

#include <libusb.h>

#define NSP_DEFAULT_CONFIG 1
#define NSP_DEFAULT_IFACE 0
#define NSP_TIMEOUT 10000

static libusb_context * usb_ctx = NULL;

int usb_init() {
	if (usb_ctx)
		return NSPIRE_ERR_SUCCESS;

	if (libusb_init(&usb_ctx))
		return -NSPIRE_ERR_LIBUSB;

	//libusb_set_debug(usb_ctx, 3);

	return NSPIRE_ERR_SUCCESS;
}

void usb_finish() {
	libusb_exit(usb_ctx);
}

int usb_get_device(usb_device_t *handle, libusb_device_handle *dev) {
	int i;
	struct libusb_config_descriptor *config;
	const struct libusb_interface_descriptor *iface;

	/* Libusb boilerplate */
	if (libusb_set_configuration(dev, NSP_DEFAULT_CONFIG))
		goto error_close;
	if (libusb_reset_device(dev))
		goto error_close;
	if (libusb_claim_interface(dev, NSP_DEFAULT_IFACE))
		goto error_close;

	/* Find and use first found input and output endpoints */
	/* We can't hardcode or else it won't work in recovery mode */
	if (libusb_get_active_config_descriptor(
			libusb_get_device(dev), &config))
		goto error_close;
	if (config->bNumInterfaces < NSP_DEFAULT_IFACE)
		goto error_free_desc;
	iface = config->interface[NSP_DEFAULT_IFACE].altsetting;

	handle->ep_in = 0;
	handle->ep_out = 0;
	for (i=0; i<iface->bNumEndpoints; i++) {
		unsigned char ep = iface->endpoint[i].bEndpointAddress;
		if (ep & LIBUSB_ENDPOINT_IN) {
			if (!handle->ep_in) handle->ep_in = ep;
		} else {
			if (!handle->ep_out) handle->ep_out = ep;
		}
	}
	libusb_free_config_descriptor(config);

	if (!handle->ep_in || !handle->ep_out)
		goto error_close;

	handle->dev = dev;
	return NSPIRE_ERR_SUCCESS;
error_free_desc:
	libusb_free_config_descriptor(config);
error_close:
error:
	return -NSPIRE_ERR_NODEVICE;
}

void usb_free_device(usb_device_t *handle) {
	libusb_release_interface(handle->dev, NSP_DEFAULT_IFACE);
}

static inline int usb_xfer(libusb_device_handle *handle, unsigned char ep,
		void *ptr, int len) {
	int ret, transferred = 0;
	ret = libusb_bulk_transfer(handle,
		ep,
		ptr,
		len,
		&transferred,
		NSP_TIMEOUT);

	switch (ret) {
	case 0:				return (len - transferred);
	case LIBUSB_ERROR_NO_DEVICE:	return -NSPIRE_ERR_NODEVICE;
	case LIBUSB_ERROR_TIMEOUT:	return -NSPIRE_ERR_TIMEOUT;
	default:			return -NSPIRE_ERR_LIBUSB;
	}
}

int usb_write(usb_device_t *handle, void *ptr, int len) {
	return usb_xfer(handle->dev, handle->ep_out, ptr, len);
}

int usb_read(usb_device_t *handle, void *ptr, int len) {
	return usb_xfer(handle->dev, handle->ep_in, ptr, len);
}

