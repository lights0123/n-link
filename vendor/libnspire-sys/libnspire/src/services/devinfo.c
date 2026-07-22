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
#include "endianconv.h"
#include "devinfo.h"

int nspire_device_info(nspire_handle_t *handle, struct nspire_devinfo *i) {
	int ret;
	uint8_t buffer[253];
	char *devname, *file, *os;
	struct deviceinfo_01 devinfo;

	if ( (ret = service_connect(handle, 0x4020)) )
		return ret;

	if ( (ret = data_write8(handle, 0x01)) )
		goto end;

	if ( (ret = data_read(handle, buffer, sizeof(buffer), NULL)) )
		goto end;

	/* Shift up one byte to align */
	memmove(&devinfo, buffer+1, sizeof(devinfo));

	/* Maybe we should convert all this to a data scan call */
	i->storage.free		= dcpu64(devinfo.flash_free);
	i->storage.total	= dcpu64(devinfo.flash_total);
	i->ram.free		= dcpu64(devinfo.ram_free);
	i->ram.total		= dcpu64(devinfo.ram_total);

	i->versions[NSPIRE_VER_OS].major = devinfo.p_version[0];
	i->versions[NSPIRE_VER_OS].minor = devinfo.p_version[1];
	i->versions[NSPIRE_VER_OS].build =
		devinfo.p_version[2]<<8 | devinfo.p_version[3];

	i->versions[NSPIRE_VER_BOOT1].major = devinfo.boot1_version[0];
	i->versions[NSPIRE_VER_BOOT1].minor = devinfo.boot1_version[1];
	i->versions[NSPIRE_VER_BOOT1].build =
		devinfo.boot1_version[2]<<8 | devinfo.boot1_version[3];

	i->versions[NSPIRE_VER_BOOT2].major = devinfo.boot2_version[0];
	i->versions[NSPIRE_VER_BOOT2].minor = devinfo.boot2_version[1];
	i->versions[NSPIRE_VER_BOOT2].build =
		devinfo.boot2_version[2]<<8 | devinfo.boot2_version[3];

	i->hw_type		= devinfo.device;

	i->batt.status		= devinfo.batt_lvl;
	i->batt.is_charging	= devinfo.is_charging;
	i->clock_speed		= devinfo.clock_speed;

	i->lcd.width		= dcpu16(devinfo.lcd_width);
	i->lcd.height		= dcpu16(devinfo.lcd_height);
	i->lcd.bbp		= devinfo.lcd_bbp;
	i->lcd.sample_mode	= devinfo.lcd_sample_mode;

	memcpy(i->electronic_id, devinfo.full_electronic_id,
			sizeof(i->electronic_id) - 1);
	i->electronic_id[sizeof(i->electronic_id) - 1] = '\0';

	i->runlevel		= dcpu16(devinfo.run_level);

	if ( (ret = data_write8(handle, 0x02)) )
		goto end;

	if ( (ret = data_read(handle, buffer, sizeof(buffer), NULL)) )
		goto end;

	if ( (ret = data_scan("bS", buffer, sizeof(buffer),
			NULL, &devname)) )
		goto end;

	strncpy(i->device_name, devname, sizeof(i->device_name)-1);
	i->device_name[sizeof(i->device_name)-1] = '\0';

	if ( (ret = data_write8(handle, 0x03)) )
		goto end;

	if ( (ret = data_read(handle, buffer, sizeof(buffer), NULL)) )
		goto end;

	if ( (ret = data_scan("bSS", buffer, sizeof(buffer),
			NULL, &file, &os)) )
		goto end;

	strncpy(i->extensions.file, file, sizeof(i->extensions.file)-1);
	strncpy(i->extensions.os, os, sizeof(i->extensions.os)-1);

	i->extensions.file[sizeof(i->extensions.file)-1] = '\0';
	i->extensions.os[sizeof(i->extensions.os)-1] = '\0';

	ret = NSPIRE_ERR_SUCCESS;

end:
	service_disconnect(handle);
	return ret;
}
