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

#include <string.h>

#include "handle.h"

#include "error.h"
#include "data.h"
#include "service.h"

typedef void (*nspire_callback)(size_t, void*);
int nspire_os_send(nspire_handle_t *handle, void* data, size_t size, nspire_callback cb, void *cb_data) {
	int ret;
	size_t len;
	uint8_t buffer[sizeof(struct packet)], *ptr = data;

	if ( (ret = service_connect(handle, 0x4080)) )
		return ret;

	if ( (ret = data_build("bw", buffer, sizeof(buffer), &len,
			0x03, (uint32_t)size)) )
		goto end;

	if ( (ret = data_write(handle, buffer, len)) )
		goto end;

	if ( (ret = data_read(handle, buffer, sizeof(buffer), NULL)) )
		goto end;

	if (buffer[0] != 0x04) {
		ret = -NSPIRE_ERR_OSFAILED;
		goto end;
	}

	size_t datasize = packet_max_datasize(handle) - 1;
	buffer[0] = 0x05;
	while (size) {
		len = (datasize < size) ? datasize : size;

		memcpy(buffer + 1, ptr, len);
		if ( (ret = data_write(handle, buffer, len+1)) )
			goto end;

		if (ptr == data) {
			/* First run - read 0xFF00 */
			uint16_t code;
			if ( (ret = data_read(handle, &code, 2, NULL)) )
				goto end;

			if (dcpu16(code) != 0xFF00 && dcpu16(code) != 0x0400) {
				ret = -NSPIRE_ERR_OSFAILED;
				goto end;
			}
		}

		size -= len;
		ptr += len;
		cb(size, cb_data);
	}

	while (1) {
		if ( (ret = data_read(handle, buffer, sizeof(buffer), NULL)) )
			goto end;

		if (buffer[0] == 0xFF) {
			ret = -NSPIRE_ERR_OSFAILED;
			goto end;
		}

		// Yes, over 100% is actually possible...
		if (buffer[1] >= 100)
			break;
	}

	ret = NSPIRE_ERR_SUCCESS;
end:
	service_disconnect(handle);
	return ret;
}
