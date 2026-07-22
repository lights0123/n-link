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
#include "dir.h"

static int dir_enum(nspire_handle_t *handle, struct nspire_dir_info **d) {
	int ret;
	char *name;
	uint32_t size, date;
	uint8_t is_dir;
	struct nspire_dir_info *new_dir;
	struct nspire_dir_item *current;

	unsigned char buffer[254];

	if ( (ret = data_write8(handle, 0x0E)) )
		return ret;
	if ( (ret = data_read(handle, buffer, sizeof(buffer), NULL)) )
		return ret;

	if (buffer[0] == 0xFF)
		return 1;

	if ( (ret = data_scan("hbswwb0", buffer, sizeof(buffer),
			NULL, NULL, &name, &size, &date, &is_dir)) )
		return ret;

	new_dir = realloc(*d, sizeof(struct nspire_dir_info) +
			(((*d)->num + 1) * sizeof(struct nspire_dir_item)));
	if (!new_dir)
		return -NSPIRE_ERR_NOMEM;
	current = new_dir->items + new_dir->num;
	new_dir->num++;
	*d = new_dir;

	strncpy(current->name, name, sizeof(current->name));
	current->name[sizeof(current->name)-1] = '\0';
	current->size = size;
	current->date = date;
	current->type = is_dir;

	return NSPIRE_ERR_SUCCESS;
}

int nspire_dirlist(nspire_handle_t *handle, const char *path,
		struct nspire_dir_info **info_ptr) {
	int ret;
	size_t len;
	uint8_t buffer[254], code;
	uint16_t result;
	struct nspire_dir_info *d;

	if ( (ret = service_connect(handle, 0x4060)) )
		return ret;

	/* Begin dir enum */
	if ( (ret = data_build("bs0", buffer, sizeof(buffer), &len,
			0x0D, path)) )
		goto end;

	if ( (ret = data_write(handle, buffer, len)) )
		goto end;

	if ( (ret = data_read(handle, buffer, 2, NULL)) )
		goto end;

	if ( (ret = data_scan("bb", buffer, sizeof(buffer),
			NULL, &code)) )
		goto end;

	switch (code) {
	case 0x0A:
		ret = -NSPIRE_ERR_NONEXIST;
		goto end;
	case 0x0F:
		ret = -NSPIRE_ERR_INVALID;
		goto end;
	}

	d = malloc(sizeof(struct nspire_dir_info));
	(d)->num = 0;
	if (!d) {
		ret = -NSPIRE_ERR_NOMEM;
		goto end;
	}

	*info_ptr = d;
	/* Start enumerating */
	while (1) {
		ret = dir_enum(handle, info_ptr);
		if (ret < 0) {
			free(*info_ptr);
			goto end;
		}

		if (ret)
			break;
	}

	/* End dir enum */
	if ( (ret = data_build("b", buffer, sizeof(buffer), &len,
			0x0F)) )
		goto end;

	if ( (ret = data_write(handle, buffer, len)) )
		goto end;

	if ( (ret = data_read(handle, buffer, 2, NULL)) )
		goto end;

	if ( (ret = data_scan("h", buffer, sizeof(buffer),
			NULL, &result)) )
		goto end;

	// TODO: result ignored here, what to do on failure anyway?

	ret = NSPIRE_ERR_SUCCESS;
end:
	service_disconnect(handle);
	return ret;
}

void nspire_dirlist_free(struct nspire_dir_info *d) {
	free(d);
}

int nspire_dir_create(nspire_handle_t *handle, const char *path) {
	int ret;
	size_t len;
	uint16_t result;
	uint8_t buffer[254];

	if ( (ret = service_connect(handle, 0x4060)) )
		return ret;

	if ( (ret = data_build("hs", buffer, sizeof(buffer), &len,
			0x0A03, path)) )
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

int nspire_dir_delete(nspire_handle_t *handle, const char *path) {
	int ret;
	size_t len;
	uint16_t result;
	uint8_t buffer[254];


	if ( (ret = service_connect(handle, 0x4060)) )
		return ret;

	if ( (ret = data_build("hs", buffer, sizeof(buffer), &len,
			0x0B03, path)) )
		goto end;

	if ( (ret = data_write(handle, &buffer, len)) )
		goto end;

	if ( (ret = data_read(handle, &buffer, 2, NULL)) )
		goto end;

	if ( (ret = data_scan("h", buffer, sizeof(buffer), &result)) )
		goto end;

	ret = (result == 0xFF00) ? NSPIRE_ERR_SUCCESS : -NSPIRE_ERR_NONEXIST;
end:
	service_disconnect(handle);
	return ret;
}

int nspire_attr(nspire_handle_t *handle, const char *path,
		struct nspire_dir_item *info) {
	int ret;
	size_t len;
	uint8_t is_dir, buffer[254];
	uint32_t size, date;

	if ( (ret = service_connect(handle, 0x4060)) )
		return ret;

	if ( (ret = data_build("hs0", buffer, sizeof(buffer), &len,
			0x2001, path)) )
		goto end;

	if ( (ret = data_write(handle, buffer, len)) )
		goto end;

	if ( (ret = data_read(handle, buffer, sizeof(buffer), NULL)) )
		goto end;

	if (buffer[0] != 0x20) {
		ret = -NSPIRE_ERR_NONEXIST;
		goto end;
	}

	if ( (ret = data_scan("bwwb0", buffer, sizeof(buffer),
			NULL, &size, &date, &is_dir)) )
		goto end;

	strncpy(info->name, path, sizeof(info->name));
	info->name[sizeof(info->name)-1] = '\0';
	info->size = size;
	info->date = date;
	info->type = is_dir;

	ret = NSPIRE_ERR_SUCCESS;
end:
	service_disconnect(handle);
	return ret;
}
