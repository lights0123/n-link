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

#ifndef NSP_SCREENSHOT_H
#define NSP_SCREENSHOT_H

#include <inttypes.h>
#include "handle.h"

struct nspire_image {
	uint16_t width, height;
	uint8_t bbp;

	unsigned char data[];
};

int nspire_screenshot(nspire_handle_t *handle, struct nspire_image **ptr);

#endif
