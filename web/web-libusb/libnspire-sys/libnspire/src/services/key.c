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

#include <stdlib.h>
#include <string.h>

#include "handle.h"
#include "error.h"
#include "data.h"
#include "service.h"
#include "key.h"

int nspire_send_key(nspire_handle_t *handle, uint32_t code) {
	int ret;
	uint8_t buffer[26];

	if ((ret = service_connect(handle, 0x4042)))
		return ret;

	buffer[0] = 1;
	buffer[1] = 0;
	buffer[2] = 0;
	buffer[3] = 0x80;
	if ((ret = data_write(handle, buffer, 4)))
		goto end;

	memset((void *)buffer, 0, sizeof(buffer));
	buffer[4] = 0x08;
	buffer[5] = 0x02;
	buffer[6] = (uint8_t)(code >> 16);
	buffer[8] = (uint8_t)(code >> 8);
	buffer[24] = (uint8_t)(code & 0xFF);
	if ((ret = data_write(handle, buffer, sizeof(buffer))))
		goto end;

end:
	service_disconnect(handle);
	return ret;
}