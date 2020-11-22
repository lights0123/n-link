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
#include <stdarg.h>

#include "endianconv.h"
#include "error.h"
#include "packet.h"
#include "handle.h"

static int handle_unknown(nspire_handle_t *handle, struct packet p) {
	int ret;

	switch (p.dst_sid) {
	case 0x40de: /* Disconnection */
		ret = packet_ack(handle, p);
		break;
	default:
		ret = packet_nack(handle, p);
		break;
	}
	return ret;
}

int data_write_special(nspire_handle_t *handle, void *ptr, size_t len,
		void (*packet_callback)(struct packet *p)) {
	int ret;
	struct packet p = packet_new(handle);

	if(len < 0xFF) {
		memcpy(p.data, ptr, len);
		p.data_size = len;
	} else if(len <= packet_max_datasize(handle)) {
		memcpy(p.bigdata, ptr, len);
		p.bigdatasize = dcpu32(len);
		p.data_size = 0xFF;
	} else
		return NSPIRE_ERR_NOMEM;

	if (packet_callback)
		packet_callback(&p);

	if ((ret = packet_send(handle, p)))
		return ret;

	// Acks are handled a level lower only, in packet_send
	if (handle->is_cx2)
		return NSPIRE_ERR_SUCCESS;

	/*
	 * Wait for an ACK packet while also NACKing spurious packets (like
	 * the LOGIN request).
	 *
	 * Legitimate packets that need ACKing will be resent by the device
	 * ready for reading in the next data_read call
	 */
	while (1) {
		if ( (ret = packet_recv(handle, &p)) )
			return ret;

		if (p.dst_sid == handle->host_sid) {
			if ((p.src_sid == 0xFF || p.src_sid == 0xFE)) {
				handle->seq++;
				if (!handle->seq) handle->seq++;
				return NSPIRE_ERR_SUCCESS;
			} else
			if (p.src_sid == 0xD3) {
				return -NSPIRE_ERR_INVALPKT;
			}
		} else {
			handle_unknown(handle, p);
		}
	}
}

int data_write(nspire_handle_t *handle, void *ptr, size_t len) {
	return data_write_special(handle, ptr, len, NULL);
}

int data_read(nspire_handle_t *handle, void *ptr, size_t maxlen, size_t *actual) {
	int ret;
	size_t len;
	struct packet p;

	while (1) {
		if ( (ret = packet_recv(handle, &p)) )
			return ret;

		if (p.dst_sid == handle->host_sid) {
			// Acks are handled a level lower only, in packet_recv
			if(handle->is_cx2)
				break;

			if ( (ret = packet_ack(handle, p)) )
				return ret;
			break;
		} else {
			handle_unknown(handle, p);
		}

		if (ret)
			return ret;
	}

	len = (maxlen < packet_datasize(&p)) ? maxlen : packet_datasize(&p);
	memcpy(ptr, packet_dataptr(&p), len);

	if(actual)
		*actual = len;

	return NSPIRE_ERR_SUCCESS;
}

/*
	Format string:
		b	8 bit integer
		h	16 bit integer
		w	32 bit integer
		l	64 bit integer
		0	expect zero
		s	null terminated string len >= 8
		S	null terminated string
*/

int data_scan(const char *format, const void *buffer, size_t len, ...) {
	int ret;
	const uint8_t *ptr = buffer, *end = ptr + len;
	va_list ap;

	va_start(ap, len);
	while (ptr < end && *format) {
		len = end - ptr;
		switch (*format) {
		case 'b':
		{
			uint8_t *in = va_arg(ap, uint8_t*);
			if (in) *in = *ptr;
			ptr++;
			break;
		}
		case 'h':
		{
			uint16_t *in = va_arg(ap, uint16_t*);
			if (len < 2) {
				ret = -NSPIRE_ERR_INVALID;
				goto end;
			}
			if (in) {
				*in = ptr[0]<<8 | ptr[1];
			}
			ptr+=2;
			break;

		}
		case 'w':
		{
			uint32_t *in = va_arg(ap, uint32_t*);
			if (len < 4) {
				ret = -NSPIRE_ERR_INVALID;
				goto end;
			}
			if (in) {
				*in = 	ptr[0]<<24 |
					ptr[1]<<16 |
					ptr[2]<<8  |
					ptr[3];
			}
			ptr+=4;
			break;

		}
		case 'l':
		{
			uint64_t *in = va_arg(ap, uint64_t*);
			if (len < 8) {
				ret = -NSPIRE_ERR_INVALID;
				goto end;
			}
			if (in) {
				uint32_t low, high;
				high =	ptr[0]<<24 |
					ptr[1]<<16 |
					ptr[2]<<8  |
					ptr[3];
				low =	ptr[4]<<24 |
					ptr[5]<<16 |
					ptr[6]<<8  |
					ptr[7];
				*in =	((uint64_t)high<<32) | low;

			}
			ptr+=8;
			break;
		}
		case 's':
		{
			char **in = va_arg(ap, char **);
			uint8_t *next = memchr(ptr, '\0', len);

			if (!next) {
				ret = -NSPIRE_ERR_INVALID;
				goto end;
			}
			*in = (char*)ptr;

			/* Strings are padded with zeros until len >= 8 */
			/* Take this into account */
			if (next - ptr < 8) {
				ptr = next + (8 - (next - ptr));
			} else {
				ptr = next;
			}
			ptr++;
			break;
		}
		case 'S':
		{
			char **in = va_arg(ap, char **);
			uint8_t *next = memchr(ptr, '\0', len);

			if (!next) {
				ret = -NSPIRE_ERR_INVALID;
				goto end;
			}
			*in = (char*)ptr;

			ptr = next + 1;
			break;
		}
		case '0':
			if (*ptr != 0) {
				ret = -NSPIRE_ERR_INVALID;
				goto end;
			}
			ptr++;
			break;
		default:
			ret = -NSPIRE_ERR_INVALID;
			goto end;
		}
		format++;
	}

	ret = NSPIRE_ERR_SUCCESS;
end:
	va_end(ap);
	return ret;
}

int data_build(const char *format, void *buffer,
		size_t len, size_t *wrote_bytes, ...) {
	int ret;
	uint8_t *ptr = buffer, *end = ptr + len;
	va_list ap;

	va_start(ap, wrote_bytes);
	while (ptr < end && *format) {
		len = end - ptr;
		switch (*format) {
		case 'b':
		{
			uint8_t in = va_arg(ap, unsigned int);
			*ptr = in;
			ptr++;
			break;
		}
		case 'h':
		{
			uint16_t in = va_arg(ap, unsigned int);
			if (len < 2) {
				ret = -NSPIRE_ERR_INVALID;
				goto end;
			}
			ptr[0] = (in>>8)	& 0xFF;
			ptr[1] = (in)		& 0xFF;
			ptr+=2;
			break;

		}
		case 'w':
		{
			uint32_t in = va_arg(ap, uint32_t);
			if (len < 4) {
				ret = -NSPIRE_ERR_INVALID;
				goto end;
			}
			ptr[0] = (in>>24)	& 0xFF;
			ptr[1] = (in>>16)	& 0xFF;
			ptr[2] = (in>>8)	& 0xFF;
			ptr[3] = (in)		& 0xFF;
			ptr+=4;
			break;

		}
		case 'l':
		{
			uint64_t in = va_arg(ap, uint64_t);
			if (len < 8) {
				ret = -NSPIRE_ERR_INVALID;
				goto end;
			}
			ptr[0] = (in>>56)	& 0xFF;
			ptr[1] = (in>>48)	& 0xFF;
			ptr[2] = (in>>40)	& 0xFF;
			ptr[3] = (in>>32)	& 0xFF;
			ptr[4] = (in>>24)	& 0xFF;
			ptr[5] = (in>>16)	& 0xFF;
			ptr[6] = (in>>8)	& 0xFF;
			ptr[7] = (in)		& 0xFF;
			ptr+=8;
			break;
		}
		case 's':
		{
			char *in = va_arg(ap, char *);
			size_t strl = strlen(in);

			if (len <= strl || len <= 9) {
				ret = -NSPIRE_ERR_INVALID;
				goto end;
			}

			/* Strings are padded with zeros until len (including the final 0) >= 9 */
			/* Take this into account */
			memset(ptr, 0, 9);
			strncpy((char*)ptr, in, len);

			if (strl < 8) {
				ptr += 8;
			} else {
				ptr += strl;
			}
			*ptr++ = '\0';
			break;
		}
		case 'S':
		{
			char *in = va_arg(ap, char *);
			size_t strl = strlen(in);

			if (len <= strl) {
				ret = -NSPIRE_ERR_INVALID;
				goto end;
			}

			strncpy((char*)ptr, in, len);
			ptr += strl;
			*ptr++ = '\0';
			break;
		}
		case '0':
			*ptr++ = '\0';
			break;
		default:
			ret = -NSPIRE_ERR_INVALID;
			goto end;
		}
		format++;
	}

	if (wrote_bytes)
		*wrote_bytes = ptr - (uint8_t*)buffer;

	ret = NSPIRE_ERR_SUCCESS;
end:
	va_end(ap);
	return ret;
}
