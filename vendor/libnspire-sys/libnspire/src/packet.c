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

#include <inttypes.h>
#include <stddef.h>

#include "cx2.h"
#include "handle.h"
#include "packet.h"
#include "error.h"
#include "usb.h"
#include "endianconv.h"

#define PACKET_CONSTANT 0x54FD
#define HEADER_SIZE offsetof(struct packet, data)

static uint8_t calculate_header_checksum(uint8_t *data, uint8_t size) {
	uint8_t chksum = 0;
	int i;
	for (i=0; i<size; i++) chksum += data[i];
	return chksum;
}

static uint16_t calculate_data_checksum(uint8_t *data, uint32_t size) {
	uint16_t chksum = 0;
	int i;
	for (i=0; i<size; i++) {
		uint16_t tmp1, tmp2, tmp3;
		tmp1 = data[i]<<8 | chksum>>8;
		chksum &= 0xFF;
		tmp2 = (((chksum & 0xF) << 4) ^ chksum) << 8;
		tmp3 = tmp2 >> 5;
		chksum = tmp3 >> 7;
		chksum ^= tmp1 ^ tmp2 ^ tmp3;
	}
	return chksum;
}

static void fix_endian(struct packet *p) {
	p->magic			= dcpu16(p->magic);
	p->src_addr			= dcpu16(p->src_addr);
	p->src_sid			= dcpu16(p->src_sid);
	p->dst_addr			= dcpu16(p->dst_addr);
	p->dst_sid			= dcpu16(p->dst_sid);
	p->data_checksum		= dcpu16(p->data_checksum);
}

static void finalize_packet(struct packet *p) {
	p->magic = PACKET_CONSTANT;

	/* Calculate checksums */
	p->data_checksum = calculate_data_checksum(p->fulldata, packet_fulldatasize(p));
	p->header_checksum = calculate_header_checksum(
		(uint8_t*)p,
		HEADER_SIZE - 1
	);

	/* Convert endian */
	fix_endian(p);
}

static int is_header_valid(struct packet *p) {
	uint8_t header_chksum = calculate_header_checksum(
		(uint8_t*)p,
		HEADER_SIZE - 1
	);

	if (p->magic != PACKET_CONSTANT)		return 0;
	if (p->header_checksum != header_chksum)	return 0;
	return 1;
}

static int is_data_valid(struct packet *p) {
	uint16_t data_chksum = calculate_data_checksum(p->fulldata, packet_fulldatasize(p));
	if (!(p->data_checksum == data_chksum))		return 0;
	return 1;
}

#ifdef DEBUG
#include <stdio.h>
static void dump_packet(const struct packet *p) {
	printf(
		"magic		= %04x\n"
		"src_addr	= %04x\n"
		"src_sid	= %04x\n"
		"dst_addr	= %04x\n"
		"dst_sid	= %04x\n"
		"data_chksm	= %04x\n"
		"size		= %02x\n"
		"ack		= %02x\n"
		"seq		= %02x\n"
		"hdr_chksm	= %02x\n"
		"data		= ",
		p->magic,
		p->src_addr,
		p->src_sid,
		p->dst_addr,
		p->dst_sid,
		p->data_checksum,
		p->data_size,
		p->ack,
		p->seq,
		p->header_checksum);

	int i;
	for (i=0; i<(packet_fulldatasize(p)); i++) {
		printf("%02x ", p->fulldata[i]);
	}
	printf("\n");
}
#endif

int packet_send(nspire_handle_t *h, struct packet p) {
	int size = HEADER_SIZE + packet_fulldatasize(&p);

#ifdef DEBUG
	printf("\nOUT ===>\n");
	dump_packet(&p);
#endif

	finalize_packet(&p);
	if(h->is_cx2)
		return packet_send_cx2(h, (char*)&p, size);
	else
		return usb_write(&h->device, (char*)&p, size);
}

int packet_recv(nspire_handle_t *h, struct packet *p) {
	int ret;
	struct packet unused;

	/* if user passes in NULL, receive packet but ignore it */
	if (!p)
		p = &unused;

	if(h->is_cx2)
		ret = packet_recv_cx2(h, (char*)p, sizeof(*p));
	else
		ret = usb_read(&h->device, p, sizeof(*p));

	if (ret < 0)
		return ret;

	fix_endian(p);
	if (!is_header_valid(p) || !is_data_valid(p))
		return -NSPIRE_ERR_INVALPKT;

#ifdef DEBUG
	printf("\nIN  <===\n");
	dump_packet(p);
#endif

	return -NSPIRE_ERR_SUCCESS;
}

struct packet packet_new(nspire_handle_t *h) {
	struct packet p = {0};

	p.src_addr = h->host_addr;
	p.dst_addr = h->device_addr;
	p.src_sid = h->host_sid;
	p.dst_sid = h->device_sid;
	p.seq = h->is_cx2 ? 0 : h->seq;

	return p;
}

static struct packet packet_new_ack(nspire_handle_t *h, struct packet p) {
	struct packet ack = packet_new(h);

	ack.src_sid = p.seq ? 0xFF : 0xFE;
	ack.dst_sid = p.src_sid;

	ack.seq = p.seq;
	ack.ack = 0x0A;

	ack.data[0] = p.dst_sid>>8;
	ack.data[1] = p.dst_sid&0xFF;
	ack.data_size = 2;

	return ack;
}

int packet_ack(nspire_handle_t *h, struct packet p) {
	struct packet ack = packet_new_ack(h, p);
	return packet_send(h, ack);
}

int packet_nack(nspire_handle_t *h, struct packet p) {
	struct packet nack = packet_new_ack(h, p);
	nack.src_sid = 0xD3;

	return packet_send(h, nack);
}

uint8_t *packet_dataptr(struct packet *p) {
	return (p->data_size == 0xFF) ? p->bigdata : p->data;
}

uint32_t packet_datasize(const struct packet *p) {
	return (p->data_size == 0xFF) ? dcpu32(p->bigdatasize) : p->data_size;
}

uint32_t packet_max_datasize(nspire_handle_t *h) {
	return h->is_cx2 ? 1440 : 254;
}

uint32_t packet_fulldatasize(const struct packet *p) {
	return (p->data_size == 0xFF) ? (dcpu32(p->bigdatasize) + 4) : p->data_size;
}
