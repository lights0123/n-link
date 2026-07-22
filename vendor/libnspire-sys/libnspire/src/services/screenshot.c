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
#include "screenshot.h"

struct rle {
	signed char len;
	unsigned char byte;
};

static void rle_decode(const void *in, void *_out,
		size_t in_size, size_t out_size) {
	size_t len;
	unsigned char *out = _out;
	const struct rle *ptr = in;

	while (in_size > 1 && out_size) {
		if (ptr->len < 0) {
			len = -(ptr->len) + 1;
			len = len < out_size ? len : out_size;
			len = len < in_size ? len : in_size;

			memcpy(out, &ptr->byte, len);

			in_size -= (1 + (len));
			ptr = (void*)((unsigned char*)ptr + (1 + (len)));
		} else {
			len = ptr->len + 1;
			len = len < out_size ? len : out_size;
			memset(out, ptr->byte, len);

			in_size -= sizeof(struct rle);
			ptr++;
		}

		out_size -= len;
		out += len;
	}
}

int nspire_screenshot(nspire_handle_t *handle, struct nspire_image **ptr) {
	int ret;
	size_t len, in_len, out_len;
	uint8_t buffer[254], bbp, *tmp = NULL, *tmp_ptr = NULL;
	uint16_t width, height;
	uint32_t size;
	struct nspire_image *i;


	if ( (ret = service_connect(handle, 0x4024)) )
		return ret;

	if ( (ret = data_write8(handle, 0x00)) )
		return ret;

	if ( (ret = data_read(handle, buffer, sizeof(buffer), NULL)) )
		goto end;

	if ( (ret = data_scan("bwhhhhbb", buffer, sizeof(buffer),
			NULL, &size, NULL, NULL, &width, &height, &bbp, NULL)) )
		goto end;

	tmp_ptr = tmp = malloc(size);
	if (!tmp) {
		ret = -NSPIRE_ERR_NOMEM;
		goto end;
	}

	in_len = size;
	out_len = (width * height * bbp) / 8;

	i = malloc(sizeof(*i) + out_len);
	if (!i) {
		ret = -NSPIRE_ERR_NOMEM;
		goto end;
	}

	i->width = width;
	i->height = height;
	i->bbp = bbp;

	while (size) {
		if ( (ret = data_read(handle, buffer, sizeof(buffer), NULL)) )
			goto error_free;

		len = 253 < size ? 253 : size;
		memcpy(tmp_ptr, buffer + 1, len);
		tmp_ptr += len;
		size -= len;
	}

	rle_decode(tmp, i->data, in_len, out_len);
	*ptr = i;
	ret = NSPIRE_ERR_SUCCESS;
	goto end;
error_free:
	if (i) free(i);
end:
	if (tmp) free(tmp);
	service_disconnect(handle);
	return ret;
}
