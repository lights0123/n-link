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

#ifndef NSP_ERROR_H
#define NSP_ERROR_H

enum {
	NSPIRE_ERR_SUCCESS,

	/* Generic */
	NSPIRE_ERR_TIMEOUT,
	NSPIRE_ERR_NOMEM,
	NSPIRE_ERR_INVALID,

	/* USB */
	NSPIRE_ERR_LIBUSB,
	NSPIRE_ERR_NODEVICE,

	/* Packet */
	NSPIRE_ERR_INVALPKT,
	NSPIRE_ERR_NACK,

	/* Service */
	NSPIRE_ERR_BUSY,

	/* File/Dir services */
	NSPIRE_ERR_EXISTS,
	NSPIRE_ERR_NONEXIST,

	/* OS install */
	NSPIRE_ERR_OSFAILED,

	/* Number of errors */
	NSPIRE_ERR_MAX
};

const char *nspire_strerror(int error);

#endif
