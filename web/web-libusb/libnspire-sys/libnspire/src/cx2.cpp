/*
 *    This file is part of libnspire.
 *
 *    libnspire is free software: you can redistribute it and/or modify
 *    it under the terms of the GNU General Public License as published by
 *    the Free Software Foundation, either version 3 of the License, or
 *    (at your option) any later version.
 *
 *    libnspire is distributed in the hope that it will be useful,
 *    but WITHOUT ANY WARRANTY; without even the implied warranty of
 *    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *    GNU General Public License for more details.
 *
 *    You should have received a copy of the GNU General Public License
 *    along with libnspire.  If not, see <http://www.gnu.org/licenses/>.
 */

#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>

#include "endianconv.h"
#ifdef _WIN32
#include <sys/timeb.h>
#include <sys/types.h>
#include <winsock2.h>

int gettimeofday(struct timeval *t, void *timezone) {
	struct _timeb timebuffer;
	_ftime( &timebuffer );
	t->tv_sec=timebuffer.time;
	t->tv_usec=1000*timebuffer.millitm;
	return 0;
}
#define PACK( __Declaration__ ) __pragma( pack(push, 1) ) __Declaration__ __pragma( pack(pop))
#else
#include <sys/time.h>
#define PACK( __Declaration__ ) __Declaration__ __attribute__((__packed__))
#endif

#include <libusb.h>

#include "cx2.h"
#include "error.h"
#include "packet.h"
// Windows...
#undef min

enum Address {
	AddrAll		= 0xFF,
	AddrMe		= 0xFE,
	AddrCalc	= 0x01
};

enum Service {
	AddrReqService  = 0x01,
	TimeService     = 0x02,
	EchoService     = 0x03,
	StreamService   = 0x04,
	TransmitService = 0x05,
	LoopbackService = 0x06,
	StatsService    = 0x07,
	UnknownService  = 0x08,
	AckFlag         = 0x80
};

// Big endian!
PACK(struct NNSEMessage {
	uint8_t 	misc;		// Unused?
	uint8_t		service;	// Service number. If bit 7 set, an ACK
	uint8_t     src;		// Address of the source
	uint8_t     dest;		// Address of the destination
	uint8_t     unknown;	// No idea
	uint8_t     reqAck;		// 0x1: Whether an ack is expected, 0x9: Not the first try
	uint16_t    length;		// Length of the packet, including this header
	uint16_t    seqno;		// Sequence number. Increases by one for every non-ACK packet.
	uint16_t    csum;		// Checksum. Inverse of the 16bit modular sum with carry added.
});

PACK(struct NNSEMessage_AddrReq {
	NNSEMessage hdr;
	uint8_t     code; // 00
	uint8_t     clientID[64];
});

PACK(struct NNSEMessage_AddrResp {
	NNSEMessage hdr;
	uint8_t     addr;
});

PACK(struct NNSEMessage_UnkResp {
	NNSEMessage hdr;
	uint8_t     noidea[2]; // 80 03
});

PACK(struct NNSEMessage_TimeReq {
	NNSEMessage hdr;
	uint8_t     code;
});

PACK(struct NNSEMessage_TimeResp {
	NNSEMessage hdr;
	uint8_t     noidea; // 80
	uint32_t    sec;
	uint64_t    frac;
	uint32_t    frac2;
});

static uint8_t *getPacketData(const NNSEMessage *c) {
	return ((uint8_t *)c) + sizeof(NNSEMessage);
}

#ifdef DEBUG
static void dumpPacket(const NNSEMessage *message)
{
	printf("Misc:   \t%02x\n", message->misc);
	printf("Service:\t%02x\n", message->service);
	printf("Dest:   \t%02x\n", message->dest);
	printf("Src:    \t%02x\n", message->src);
	printf("Unknown:\t%02x\n", message->unknown);
	printf("ReqAck: \t%02x\n", message->reqAck);
	printf("Length: \t%04x\n", ntohs(message->length));
	printf("SeqNo:  \t%04x\n", ntohs(message->seqno));
	printf("Csum:   \t%04x\n", ntohs(message->csum));

	auto datalen = ntohs(message->length) - sizeof(NNSEMessage);
	for(int i = 0; i < datalen; ++i)
		printf("%02x ", getPacketData(message)[i]);

	printf("\n");
}
#endif

static uint16_t compute_checksum(const uint8_t *data, uint32_t size)
{
	uint32_t acc = 0;

	if (size > 0)
	{
		for (uint32_t i = 0; i < size - 1; i += 2)
		{
			uint16_t cur = (((uint16_t)data[i]) << 8) | data[i + 1];
			acc += cur;
		}

		if (size & 1)
			acc += (((uint16_t)data[size - 1]) << 8);
	}

	while (acc >> 16)
		acc = (acc >> 16) + uint16_t(acc);

	return acc;
}

static bool readPacket(libusb_device_handle *handle, NNSEMessage *message, int maxlen)
{
	if(maxlen < sizeof(NNSEMessage))
		return false;

	int transferred = 0;
	memset(message, 0, sizeof(NNSEMessage));
	int r = libusb_bulk_transfer(handle, 0x81, reinterpret_cast<unsigned char*>(message), maxlen, &transferred, 60000);

	if(r < 0
		|| transferred < sizeof(NNSEMessage)) {
		printf("%d %d %d", r, transferred, 5);
		return false;
		}

	const auto completeLength = ntohs(message->length);

	if(completeLength < sizeof(NNSEMessage)
		|| completeLength > maxlen) {
		printf("complete %d", completeLength);
		return false;
		}

	uint8_t *data = reinterpret_cast<uint8_t*>(message) + transferred;
	auto remainingLength = completeLength - transferred;
	while(remainingLength > 0)
	{
		r = libusb_bulk_transfer(handle, 0x81, data, remainingLength, &transferred, 1000);
		if(r < 0) {
		printf("%d", r);
			return false;
			}

		data += transferred;
		remainingLength -= transferred;
	}

#ifdef DEBUG
	printf("Got packet:\n");
	dumpPacket(message);
#endif

	if(compute_checksum(reinterpret_cast<uint8_t*>(message), transferred) != 0xFFFF)
		return false;

	return true;
}

static bool writePacket(libusb_device_handle *handle, NNSEMessage *message)
{
	auto length = ntohs(message->length);

	message->csum = 0;
	message->csum = htons(compute_checksum(reinterpret_cast<uint8_t*>(message), length) ^ 0xFFFF);
    printf("comp");
	if(compute_checksum(reinterpret_cast<uint8_t*>(message), length) != 0xFFFF)
		return false;
    printf("comp done");
//#ifdef DEBUG
	printf("Sending packet:\n");
	dumpPacket(message);
//#endif

	int transferred = 0;
	int r = libusb_bulk_transfer(handle, 0x01, reinterpret_cast<unsigned char*>(message), length, &transferred, 1000);
	if(r < 0
		|| length != transferred)
		return false;

	return true;
}

static uint16_t nextSeqno()
{
	static uint16_t seqno = 0;
	return seqno++;
}

template <typename T> bool sendMessage(libusb_device_handle *handle, T &message)
{
	message.hdr.src = AddrMe;
	message.hdr.dest = AddrCalc;
	message.hdr.length = htons(sizeof(T));
	message.hdr.seqno = htons(nextSeqno());

	return writePacket(handle, &message.hdr);
}

template <typename T> T* messageCast(NNSEMessage *message)
{
	if(ntohs(message->length) < sizeof(T))
		return nullptr;

	return reinterpret_cast<T*>(message);
}

static void handlePacket(struct nspire_handle *nsp_handle, NNSEMessage *message, uint8_t **streamdata = nullptr, int *streamsize = nullptr)
{
	auto *handle = nsp_handle->device.dev;

	if(message->dest != AddrMe && message->dest != AddrAll)
	{
#ifdef DEBUG
		printf("Not for me?\n");
#endif
		return;
	}

	if(message->service & AckFlag)
	{
#ifdef DEBUG
		printf("Got ack for %02x\n", ntohs(message->seqno));
#endif
		return;
	}

	if(message->reqAck & 1)
	{
		NNSEMessage ack = {};
		ack.misc = message->misc;
		ack.service = uint8_t(message->service | AckFlag);
		ack.src = message->dest;
		ack.dest = message->src;
		ack.unknown = message->unknown;
		ack.reqAck = uint8_t(message->reqAck & ~1);
		ack.length = htons(sizeof(NNSEMessage));
		ack.seqno = message->seqno;

		if(!writePacket(handle, &ack))
			printf("Failed to ack\n");
	}
    printf("SERVICE %d", message->service);
	switch(message->service & ~AckFlag)
	{
		case AddrReqService:
		{
			const NNSEMessage_AddrReq *req = messageCast<NNSEMessage_AddrReq>(message);
			if(!req || req->code != 0)
				goto drop;

#ifdef DEBUG
			printf("Got request from client %s (product id %c%c)\n", &req->clientID[12], req->clientID[10], req->clientID[11]);
#endif
/*			Sending this somehow introduces issues like the time request not
			arriving or the calc responding with yet another address request.
			// Address release request. Not sure how that works.
			NNSEMessage_AddrResp resp{};
			resp.hdr.service = message->service;
			resp.addr = AddrCalc;

			if(!sendMessage(resp))
				printf("Failed to send message\n");
*/

			NNSEMessage_AddrResp resp2 = {};
			resp2.hdr.service = message->service;
			resp2.addr = 0x80; // No idea

		    // In some cases on HW and in Firebird always after reconnecting
			// it ignores the first packet for some reason. So just send it
			// twice (the seqno doesn't really matter at this point), if it
			// receives both it'll ignore the second one.
			if(!sendMessage(handle, resp2) || !sendMessage(handle, resp2))
				printf("Failed to send message\n");

			break;
		}
		case TimeService:
		{
			const NNSEMessage_TimeReq *req = messageCast<NNSEMessage_TimeReq>(message);
			if(!req || req->code != 0)
				goto drop;

#ifdef DEBUG
			printf("Got time request\n");
#endif

			struct timeval val;
			gettimeofday(&val, nullptr);

			NNSEMessage_TimeResp resp = {};
			resp.hdr.service = message->service;
			resp.noidea = 0x80;
			resp.sec = htonl(uint32_t(val.tv_sec));
			resp.frac = 0;

			if(!sendMessage(handle, resp))
				printf("Failed to send message\n");

			nsp_handle->cx2_handshake_complete = true;
			break;
		}
		case UnknownService:
		{
			if(ntohs(message->length) != sizeof(NNSEMessage) + 1 || getPacketData(message)[0] != 0x01)
				goto drop;

#ifdef DEBUG
			printf("Got packet for unknown service\n");
#endif

			NNSEMessage_UnkResp resp = {};
			resp.hdr.service = message->service;
			resp.noidea[0] = 0x81;
			resp.noidea[1] = 0x03;

			if(!sendMessage(handle, resp))
				printf("Failed to send message\n");

			break;
		}
		case StreamService:
		{
			if(streamdata)
				*streamdata = getPacketData(message);
			if(streamsize)
				*streamsize = ntohs(message->length) - sizeof(NNSEMessage);

			break;
		}
		default:
			printf("Unhandled service %02x\n", message->service & ~AckFlag);
	}

	return;

	drop:
	printf("Ignoring packet.\n");
}

static bool assureReady(struct nspire_handle *nsp_handle)
{
	if(nsp_handle->cx2_handshake_complete)
		return true;

	auto *handle = nsp_handle->device.dev;

	const int maxlen = sizeof(NNSEMessage) + 1472;
	NNSEMessage * const message = reinterpret_cast<NNSEMessage*>(malloc(maxlen));
	printf("begin loop");
	for(int i = 10; i-- && !nsp_handle->cx2_handshake_complete;)
	{
	    printf("looping partially");
		if(!readPacket(handle, message, maxlen)) {
		printf("cont");
			continue;
			}

		handlePacket(nsp_handle, message);
	}
	free(message);

	return nsp_handle->cx2_handshake_complete;
}

int packet_send_cx2(struct nspire_handle *nsp_handle, char *data, int size)
{
    printf("packet_send_cx2");
	if(!assureReady(nsp_handle))
		return -NSPIRE_ERR_BUSY;
    printf("assuredReady");
	auto *handle = nsp_handle->device.dev;

	int len = sizeof(NNSEMessage) + size;
	NNSEMessage *msg = reinterpret_cast<NNSEMessage*>(malloc(len));

	msg->service = StreamService;
	msg->src = AddrMe;
	msg->dest = AddrCalc;
	msg->reqAck = 1;
	msg->length = htons(len);
	msg->seqno = htons(nextSeqno());

	memcpy(getPacketData(msg), data, size);

	int ret = -NSPIRE_ERR_SUCCESS;
	if(!writePacket(handle, msg)) {
	    printf("failed to write packet");
		ret = -NSPIRE_ERR_BUSY;
	} else
	{
		const int maxlen = sizeof(NNSEMessage) + 1472;
		NNSEMessage * const message = reinterpret_cast<NNSEMessage*>(malloc(maxlen));

		bool acked = false;
		for(int i = 10; i-- && !ret && !acked;)
		{
			if(!readPacket(handle, message, maxlen))
				continue;

			handlePacket(nsp_handle, message);

			if(message->dest == AddrMe
				&& message->service == (StreamService | AckFlag)
				&& message->seqno == msg->seqno)
				acked = true;
		}

		if(!acked)
			ret = -NSPIRE_ERR_BUSY;

		free(message);
	}

	free(msg);

	return ret;
}

int packet_recv_cx2(struct nspire_handle *nsp_handle, char *data, int size)
{
	if(!assureReady(nsp_handle))
		return -NSPIRE_ERR_BUSY;

	auto *handle = nsp_handle->device.dev;

	const int maxlen = sizeof(NNSEMessage) + 1472;
	NNSEMessage * const message = reinterpret_cast<NNSEMessage*>(malloc(maxlen));

	uint8_t *streamdata = nullptr;
	int streamsize = 0;
	for(int i = 10; i-- && !streamdata;)
	{
		if(!readPacket(handle, message, maxlen))
			continue;

		handlePacket(nsp_handle, message, &streamdata, &streamsize);
	}

	if(streamdata)
		memcpy(data, streamdata, std::min(size, streamsize));

	free(message);

	return streamdata ? -NSPIRE_ERR_SUCCESS : -NSPIRE_ERR_INVALPKT;
}
