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

#ifndef CX2_H
#define CX2_H

#ifdef _WIN32
#include <winsock2.h>
#endif
#include <libusb.h>

#ifdef __cplusplus
extern "C" {
#endif

struct nspire_handle;

// Receive a NavNet packet wrapped in the NavNet SE protocol.
// Takes care of the handshake and other NNSE stuff like acking.
int packet_recv_cx2(struct nspire_handle *handle, char *data, int size);
// Send a NavNet packet wrapped in the NavNet SE protocol and wait for an ack.
// Takes care of the handshake and other NNSE stuff like acking.
int packet_send_cx2(struct nspire_handle *handle, char *data, int size);

#ifdef __cplusplus
}
#endif

#endif // CX2_H
