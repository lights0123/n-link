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

#ifndef NSP_DEVINFO_H
#define NSP_DEVINFO_H

#include <inttypes.h>
#include "handle.h"

enum nspire_battery {
	NSPIRE_BATT_POWERED	= 0x00,
	NSPIRE_BATT_LOW		= 0xF1,
	NSPIRE_BATT_OK		= 0x7F,
	NSPIRE_BATT_UNKNOWN	= 0xFF
};

enum nspire_version_index {
	NSPIRE_VER_OS,
	NSPIRE_VER_BOOT1,
	NSPIRE_VER_BOOT2,

	/* Reserved */
	NSPIRE_VER_MAXNUM
};

enum nspire_type {
	NSPIRE_CAS	= 0x0E,
	NSPIRE_NONCAS	= 0x1E,
	NSPIRE_CASCX	= 0x0F,
	NSPIRE_NONCASCX	= 0x1F
};

enum nspire_runlevel {
	NSPIRE_RUNLEVEL_RECOVERY	= 1,
	NSPIRE_RUNLEVEL_OS		= 2
};

struct nspire_devinfo {
	/* Flash storage */
	struct {
		uint64_t free, total;
	} storage;

	/* Memory */
	struct {
		uint64_t free, total;
	} ram;

	/* Versions */
	struct {
		uint8_t major, minor;
		uint16_t build;
	} versions[NSPIRE_VER_MAXNUM];
	enum nspire_type hw_type;

	/* Power */
	struct {
		enum nspire_battery status;
		uint8_t is_charging;
	} batt;
	uint8_t clock_speed;

	/* LCD */
	struct {
		uint16_t width, height;
		uint8_t bbp, sample_mode;
	} lcd;

	/* File extensions */
	struct {
		char file[8];
		char os[8];
	} extensions;

	/* ID */
	char device_name[20];
	char electronic_id[28];

	/* Misc */
	enum nspire_runlevel runlevel;
};

int nspire_device_info(nspire_handle_t *handle, struct nspire_devinfo *i);

#endif
