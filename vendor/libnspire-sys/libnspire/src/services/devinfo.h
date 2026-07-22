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

#ifndef _DEVINFO_H
#define _DEVINFO_H

#include "api/devinfo.h"
#ifdef _WIN32
#define PACK( ... ) __pragma( pack(push, 1) ) __VA_ARGS__ __pragma( pack(pop))
#else
#define PACK( ... ) __VA_ARGS__ __attribute__((__packed__))
#endif
PACK(struct deviceinfo_01 {
	uint64_t flash_free, flash_total;
	uint64_t ram_free, ram_total;

	uint8_t batt_lvl;

	/* Avoid unaligned accesses on some CPUs */
	/* uint16_t is_charging; */
	uint8_t __padding;
	uint8_t is_charging;

	uint8_t clock_speed;

	uint8_t p_version[4];
	uint8_t boot1_version[4];
	uint8_t boot2_version[4];

	uint32_t h_version;
	uint16_t run_level;

	uint16_t lcd_x, lcd_y, lcd_width, lcd_height;
	uint8_t lcd_bbp;
	uint8_t lcd_sample_mode;

	uint8_t device;

	uint8_t electronic_id[17];
	uint8_t full_electronic_id[27];
});

#endif
