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
int nspire_file_write(nspire_handle_t *handle, const char *path,
		void* data, size_t size, nspire_callback cb, void *cb_data) {
	int ret;
	size_t len;
	uint8_t buffer[sizeof(struct packet)], *ptr = data;
	uint16_t result;

	if ( (ret = service_connect(handle, 0x4060)) )
		return ret;

	if ( (ret = data_build("hsw", buffer, sizeof(buffer), &len,
			0x0301, path, size)) )
		goto end;

	if ( (ret = data_write(handle, buffer, len)) )
		goto end;

	if ( (ret = data_read(handle, buffer, sizeof(buffer), NULL)) )
		goto end;

	if (buffer[0] != 0x04) {
		ret = -NSPIRE_ERR_INVALID;
		goto end;
	}

	size_t datasize = packet_max_datasize(handle) - 1;

	buffer[0] = 0x05;
	while (size) {
		len = (datasize < size) ? datasize : size;

		memcpy(buffer + 1, ptr, len);
		if ( (ret = data_write(handle, buffer, len+1)) )
			goto end;

		size -= len;
		ptr += len;

		cb(size, cb_data);
	}

	if ( (ret = data_read(handle, buffer, sizeof(buffer), NULL)) )
		goto end;

	if ( (ret = data_scan("h", buffer, sizeof(buffer), &result)) )
		goto end;

	ret = (result == 0xFF00) ? NSPIRE_ERR_SUCCESS : -NSPIRE_ERR_NONEXIST;
end:
	service_disconnect(handle);
	return ret;
}

int nspire_file_read(nspire_handle_t *handle, const char *path,
		void* data, size_t size, size_t *total_bytes, nspire_callback cb, void *cb_data) {
	int ret;
	size_t len;
	uint8_t buffer[1440], *ptr = data;
	uint16_t result;
	uint32_t data_len;

	if ( (ret = service_connect(handle, 0x4060)) )
		return ret;

	if ( (ret = data_build("hs", buffer, packet_max_datasize(handle), &len,
			0x0701, path)) )
		goto end;

	if ( (ret = data_write(handle, buffer, len)) )
		goto end;

	if ( (ret = data_read(handle, buffer, packet_max_datasize(handle), NULL)) )
		goto end;

	if ( (ret = data_scan("h000000000w", buffer, packet_max_datasize(handle),
			&result, &data_len)) )
		goto end;

	if (result != 0x0301) {
		ret = -NSPIRE_ERR_NONEXIST;
		goto end;
	}

	if ( (ret = data_write8(handle, 0x04)) )
		goto end;

	if (total_bytes) *total_bytes = 0;

	size_t maxsize = packet_max_datasize(handle) - 1;

	while (data_len) {
		len = (maxsize < data_len) ? maxsize : data_len;

		if ( (ret = data_read(handle, buffer, len+1, &len)) )
			goto end;

		size_t to_copy = len - 1;
		memcpy(ptr, buffer + 1, (size < to_copy) ? size : to_copy);
		if (total_bytes) *total_bytes += (size < to_copy) ? size : to_copy;
		size -= (size < to_copy) ? size : to_copy;
		if(!size) {
			goto end;
		}

		ptr += to_copy;

		data_len -= len - 1;
		cb(size, cb_data);
	}

	if ( (ret = data_write16(handle, 0xFF00)) )
		goto end;

	ret = NSPIRE_ERR_SUCCESS;
end:
	service_disconnect(handle);
	return ret;
}

int nspire_file_move(nspire_handle_t *handle,
		const char *src, const char *dst) {
	int ret;
	size_t len;
	uint16_t result;
	uint8_t buffer[254];


	if ( (ret = service_connect(handle, 0x4060)) )
		return ret;

	if ( (ret = data_build("hss0", buffer, sizeof(buffer), &len,
			0x2101, src, dst)) )
		goto end;

	if ( (ret = data_write(handle, &buffer, len)) )
		goto end;

	if ( (ret = data_read(handle, &buffer, 2, NULL)) )
		goto end;

	if ( (ret = data_scan("h", buffer, sizeof(buffer), &result)) )
		goto end;

	ret = (result == 0xFF00) ? NSPIRE_ERR_SUCCESS : -NSPIRE_ERR_INVALID;
end:
	service_disconnect(handle);
	return ret;
}

int nspire_file_copy(nspire_handle_t *handle,
		const char *src, const char *dst) {
	int ret;
	size_t len;
	uint16_t result;
	uint8_t buffer[254];


	if ( (ret = service_connect(handle, 0x4060)) )
		return ret;

	if ( (ret = data_build("hss0", buffer, sizeof(buffer), &len,
			0x0C01, src, dst)) )
		goto end;

	if ( (ret = data_write(handle, &buffer, len)) )
		goto end;

	if ( (ret = data_read(handle, &buffer, 2, NULL)) )
		goto end;

	if ( (ret = data_scan("h", buffer, sizeof(buffer), &result)) )
		goto end;

	ret = (result == 0xFF00) ? NSPIRE_ERR_SUCCESS : -NSPIRE_ERR_INVALID;
end:
	service_disconnect(handle);
	return ret;
}

int nspire_file_delete(nspire_handle_t *handle, const char *path) {
	int ret;
	size_t len;
	uint16_t result;
	uint8_t buffer[254];


	if ( (ret = service_connect(handle, 0x4060)) )
		return ret;

	if ( (ret = data_build("hs0", buffer, sizeof(buffer), &len,
			0x0901, path)) )
		goto end;

	if ( (ret = data_write(handle, &buffer, len)) )
		goto end;

	if ( (ret = data_read(handle, &buffer, 2, NULL)) )
		goto end;

	if ( (ret = data_scan("h", buffer, sizeof(buffer), &result)) )
		goto end;

	ret = (result == 0xFF00) ? NSPIRE_ERR_SUCCESS : -NSPIRE_ERR_EXISTS;
end:
	service_disconnect(handle);
	return ret;
}


