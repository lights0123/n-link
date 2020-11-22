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

#ifndef NSP_DIR_H
#define NSP_DIR_H

#include "file.h"
#include "handle.h"

enum nspire_dir_type {
	NSPIRE_FILE	= 0,
	NSPIRE_DIR	= 1
};

struct nspire_dir_item {
	char name[240];
	uint64_t size, date;
	enum nspire_dir_type type;
};

struct nspire_dir_info {
	uint64_t num;
	struct nspire_dir_item items[];
};

int nspire_dirlist(nspire_handle_t *, const char *, struct nspire_dir_info **);
void nspire_dirlist_free(struct nspire_dir_info *d);
int nspire_dir_create(nspire_handle_t *handle, const char *path);
int nspire_dir_delete(nspire_handle_t *handle, const char *path);

int nspire_attr(nspire_handle_t *, const char *, struct nspire_dir_item *);

#define nspire_dir_move		nspire_file_move
#define nspire_dir_rename	nspire_file_move

#endif
