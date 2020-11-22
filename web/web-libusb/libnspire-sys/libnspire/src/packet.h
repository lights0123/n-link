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

#ifndef _PACKET_H
#define _PACKET_H

#include <inttypes.h>
#include "handle.h"

struct packet {
    uint16_t    magic,
                src_addr,
                src_sid,
                dst_addr,
                dst_sid,
                data_checksum;
    uint8_t     data_size, // If 0xFF, bigdata* counts
                ack,
                seq,
                header_checksum;
    union {
        uint8_t      data[254];
        struct {
            uint32_t bigdatasize;
            uint8_t  bigdata[1440];
        };
	uint8_t      fulldata[1444];
    };
};

#define packet_set_data(p, ...) do { \
		unsigned char buffer[] = { __VA_ARGS__ }; \
		p.data_size = sizeof(buffer); \
		memcpy(p.data, buffer, p.data_size); \
	} while (0)

int packet_send(nspire_handle_t *h, struct packet p);
int packet_recv(nspire_handle_t *h, struct packet *p);
struct packet packet_new(nspire_handle_t *h);
int packet_ack(nspire_handle_t *h, struct packet p);
int packet_nack(nspire_handle_t *h, struct packet p);
uint8_t *packet_dataptr(struct packet *p);
uint32_t packet_datasize(const struct packet *p);
uint32_t packet_fulldatasize(const struct packet *p);
uint32_t packet_max_datasize(nspire_handle_t *h);

#endif
