// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#include "sgx_tprotected_fs.h"
#include "sgx_tprotected_fs_t.h"
#include "protected_fs_file.h"

#include <errno.h>


static SGX_FILE* sgx_fopen_internal(const char* filename, const char* mode, const sgx_key_128bit_t *auto_key, const sgx_key_128bit_t *kdk_key)
{
	protected_fs_file* file = NULL;

	if (filename == NULL || mode == NULL)
	{
		errno = EINVAL;
		return NULL;
	}

	try {
		file = new protected_fs_file(filename, mode, auto_key, kdk_key);
	}
	catch (std::bad_alloc& e) {
		(void)e; // remove warning
		errno = ENOMEM;
		return NULL;
	}

	if (file->get_error() != SGX_FILE_STATUS_OK)
	{
		errno = file->get_error();
		delete file;
		file = NULL;
	}

	return (SGX_FILE*)file;
}


SGX_FILE* sgx_fopen_auto_key(const char* filename, const char* mode)
{
	return sgx_fopen_internal(filename, mode, NULL, NULL);
}


SGX_FILE* sgx_fopen(const char* filename, const char* mode, const sgx_key_128bit_t *key)
{
	return sgx_fopen_internal(filename, mode, NULL, key);
}


size_t sgx_fwrite(const void* ptr, size_t size, size_t count, SGX_FILE* stream)
{
	if (ptr == NULL || stream == NULL || size == 0 || count == 0)
		return 0;

	protected_fs_file* file = (protected_fs_file*)stream;

	return file->write(ptr, size, count);
}


size_t sgx_fread(void* ptr, size_t size, size_t count, SGX_FILE* stream)
{
	if (ptr == NULL || stream == NULL || size == 0 || count == 0)
		return 0;

	protected_fs_file* file = (protected_fs_file*)stream;

	return file->read(ptr, size, count);
}


int64_t sgx_ftell(SGX_FILE* stream)
{
	if (stream == NULL)
		return -1;

	protected_fs_file* file = (protected_fs_file*)stream;

	return file->tell();
}


int32_t sgx_fseek(SGX_FILE* stream, int64_t offset, int origin)
{
	if (stream == NULL)
		return -1;

	protected_fs_file* file = (protected_fs_file*)stream;

	return file->seek(offset, origin);
}


int32_t sgx_fflush(SGX_FILE* stream)
{
	if (stream == NULL)
		return EOPNOTSUPP; // TBD - currently we don't support NULL as fflush input parameter

	protected_fs_file* file = (protected_fs_file*)stream;

	return file->flush(/*false*/) == true ? 0 : EOF;
}


/* sgx_fflush_and_increment_mc
 *  Purpose: force actual write of all the cached data to the disk (see c++ fflush documentation for more details).
 *           in addition, in the first time this function is called, it adds a monotonic counter to the file
 *           in subsequent calls, the monotonic counter is incremented by one every time this function is called
 *           the monotonic counter is a limited resource, please read the SGX documentation for more details
 *
 *  Parameters:
 *      stream - [IN] the file handle (opened with sgx_fopen or sgx_fopen_auto_key)
 *
 *  Return value:
 *     int32_t  - result, 0 on success, 1 in case of an error - check sgx_ferror for error code
 *
int32_t sgx_fflush_and_increment_mc(SGX_FILE* stream)
{
	if (stream == NULL)
		return 1;

	protected_fs_file* file = (protected_fs_file*)stream;

	return file->flush(true) == true ? 0 : 1;
}
*/


int32_t sgx_ferror(SGX_FILE* stream)
{
	if (stream == NULL)
		return -1;

	protected_fs_file* file = (protected_fs_file*)stream;

	return file->get_error();
}


int32_t sgx_feof(SGX_FILE* stream)
{
	if (stream == NULL)
		return -1;
	
	protected_fs_file* file = (protected_fs_file*)stream;

	return ((file->get_eof() == true) ? 1 : 0);
}


void sgx_clearerr(SGX_FILE* stream)
{
	if (stream == NULL)
		return;

	protected_fs_file* file = (protected_fs_file*)stream;

	file->clear_error();
}


static int32_t sgx_fclose_internal(SGX_FILE* stream, sgx_key_128bit_t *key, bool import)
{
	int32_t retval = 0;

	if (stream == NULL)
		return EOF;

	protected_fs_file* file = (protected_fs_file*)stream;

	if (file->pre_close(key, import) == false)
		retval = 1;

	delete file;

	return retval;
}


int32_t sgx_fclose(SGX_FILE* stream)
{
	return sgx_fclose_internal(stream, NULL, false);
}


int32_t sgx_remove(const char* filename)
{
	return protected_fs_file::remove(filename);
}


int32_t sgx_fexport_auto_key(const char* filename, sgx_key_128bit_t *key)
{
	SGX_FILE* stream = sgx_fopen_internal(filename, "r", NULL, NULL);
	if (stream == NULL)
		return 1;

	return sgx_fclose_internal(stream, key, false);
}


int32_t sgx_fimport_auto_key(const char* filename, const sgx_key_128bit_t *key)
{
	SGX_FILE* stream = sgx_fopen_internal(filename, "r+", key, NULL);
	if (stream == NULL)
		return 1;

	return sgx_fclose_internal(stream, NULL, true);
}


int32_t sgx_fclear_cache(SGX_FILE* stream)
{
	if (stream == NULL)
		return 1;

	protected_fs_file* file = (protected_fs_file*)stream;

	return file->clear_cache();
}


// Add for MesaTEE
int32_t sgx_get_current_meta_gmac(SGX_FILE* stream, sgx_aes_gcm_128bit_tag_t out_gmac)
{
	if (stream == NULL)
		return 1;
	protected_fs_file* file = (protected_fs_file*)stream;
	return file->get_current_meta_gmac(out_gmac);
}

