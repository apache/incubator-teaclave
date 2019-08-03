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

#include "protected_fs_file.h"
#ifdef NON_SGX_PROTECTED_FS
#include "non_sgx_protected_fs.h"
#else
#include <tseal_migration_attr.h>
#include <sgx_utils.h>
#include <sgx_trts.h>
#endif


#define MASTER_KEY_NAME      "SGX-PROTECTED-FS-MASTER-KEY"
#define RANDOM_KEY_NAME      "SGX-PROTECTED-FS-RANDOM-KEY"
#define METADATA_KEY_NAME    "SGX-PROTECTED-FS-METADATA-KEY"

#define MAX_LABEL_LEN 64

typedef struct {
	uint32_t index;
	char label[MAX_LABEL_LEN];
	uint64_t node_number; // context 1
	union { // context 2
		sgx_cmac_128bit_tag_t nonce16;
		sgx_key_id_t nonce32;
	};
	uint32_t output_len; // in bits
} kdf_input_t;

#define MAX_MASTER_KEY_USAGES 65536

bool protected_fs_file::generate_secure_blob(sgx_aes_gcm_128bit_key_t* key, const char* label, uint64_t physical_node_number, sgx_aes_gcm_128bit_tag_t* output)
{
	kdf_input_t buf = {0, "", 0, "", 0};

	uint32_t len = (uint32_t)strnlen(label, MAX_LABEL_LEN + 1);
	if (len > MAX_LABEL_LEN)
	{
		last_error = EINVAL;
		return false;
	}

	// index
	// SP800-108:
	// i - A counter, a binary string of length r that is an input to each iteration of a PRF in counter mode [...].
	buf.index = 0x01;

	// label
	// SP800-108:
	// Label - A string that identifies the purpose for the derived keying material, which is encoded as a binary string. 
	//         The encoding method for the Label is defined in a larger context, for example, in the protocol that uses a KDF.
	strncpy(buf.label, label, len);

	// context and nonce
	// SP800-108: 
	// Context - A binary string containing the information related to the derived keying material.
	//           It may include identities of parties who are deriving and / or using the derived keying material and, 
	//           optionally, a nonce known by the parties who derive the keys.
	buf.node_number = physical_node_number;

	sgx_status_t status = sgx_read_rand((unsigned char*)&(buf.nonce16), sizeof(sgx_cmac_128bit_tag_t));
	if (status != SGX_SUCCESS)
	{
		last_error = status;
		return false;
	}

	// length of output (128 bits)
	buf.output_len = 0x80;

	status = sgx_rijndael128_cmac_msg(key, (const uint8_t*)&buf, sizeof(kdf_input_t), output);
	if (status != SGX_SUCCESS)
	{
		last_error = status;
		return false;
	}

	memset_s(&buf, sizeof(kdf_input_t), 0, sizeof(kdf_input_t));

	return true;
}


bool protected_fs_file::generate_secure_blob_from_user_kdk(bool restore)
{
	kdf_input_t buf = {0, "", 0, "", 0};
	sgx_status_t status = SGX_SUCCESS;

	// index
	// SP800-108:
	// i - A counter, a binary string of length r that is an input to each iteration of a PRF in counter mode [...].
	buf.index = 0x01;

	// label
	// SP800-108:
	// Label - A string that identifies the purpose for the derived keying material, which is encoded as a binary string. 
	//         The encoding method for the Label is defined in a larger context, for example, in the protocol that uses a KDF.
	strncpy(buf.label, METADATA_KEY_NAME, strlen(METADATA_KEY_NAME));

	// context and nonce
	// SP800-108: 
	// Context - A binary string containing the information related to the derived keying material.
	//           It may include identities of parties who are deriving and / or using the derived keying material and, 
	//           optionally, a nonce known by the parties who derive the keys.
	buf.node_number = 0;

	// use 32 bytes here just for compatibility with the seal key API
	if (restore == false)
	{
		status = sgx_read_rand((unsigned char*)&(buf.nonce32), sizeof(sgx_key_id_t));
		if (status != SGX_SUCCESS)
		{
			last_error = status;
			return false;
		}
	}
	else
	{
		memcpy(&buf.nonce32, &file_meta_data.plain_part.meta_data_key_id, sizeof(sgx_key_id_t));
	}
	

	// length of output (128 bits)
	buf.output_len = 0x80;

	status = sgx_rijndael128_cmac_msg(&user_kdk_key, (const uint8_t*)&buf, sizeof(kdf_input_t), &cur_key);
	if (status != SGX_SUCCESS)
	{
		last_error = status;
		return false;
	}

	if (restore == false)
	{
		memcpy(&file_meta_data.plain_part.meta_data_key_id, &buf.nonce32, sizeof(sgx_key_id_t));
	}

	memset_s(&buf, sizeof(kdf_input_t), 0, sizeof(kdf_input_t));

	return true;
}


bool protected_fs_file::init_session_master_key()
{
	sgx_aes_gcm_128bit_key_t empty_key = {0};
		
	if (generate_secure_blob(&empty_key, MASTER_KEY_NAME, 0, (sgx_aes_gcm_128bit_tag_t*)&session_master_key) == false)
		return false;

	master_key_count = 0;

	return true;
}


bool protected_fs_file::derive_random_node_key(uint64_t physical_node_number)
{
	if (master_key_count++ > MAX_MASTER_KEY_USAGES)
	{
		if (init_session_master_key() == false)
			return false;
	}

	if (generate_secure_blob(&session_master_key, RANDOM_KEY_NAME, physical_node_number, (sgx_aes_gcm_128bit_tag_t*)&cur_key) == false)
		return false;

	return true;
}


bool protected_fs_file::generate_random_meta_data_key()
{
	if (use_user_kdk_key == 1)
	{
		return generate_secure_blob_from_user_kdk(false);
	}

#ifdef NON_SGX_PROTECTED_FS
	assert(0);
	return false;
#else
	// derive a random key from the enclave sealing key	
	sgx_key_request_t key_request;
	memset(&key_request, 0, sizeof(sgx_key_request_t)); 
		
	key_request.key_name = SGX_KEYSELECT_SEAL;
	key_request.key_policy = SGX_KEYPOLICY_MRSIGNER;

	memcpy(&key_request.cpu_svn, &report.body.cpu_svn, sizeof(sgx_cpu_svn_t));
	memcpy(&key_request.isv_svn, &report.body.isv_svn, sizeof(sgx_isv_svn_t));

    key_request.attribute_mask.flags = TSEAL_DEFAULT_FLAGSMASK;
    key_request.attribute_mask.xfrm = 0x0;

	key_request.misc_mask = TSEAL_DEFAULT_MISCMASK;
		
	sgx_status_t status = sgx_read_rand((unsigned char*)&key_request.key_id, sizeof(sgx_key_id_t));
	if (status != SGX_SUCCESS)
	{
		last_error = status;
		return false;
	}
	
	status = sgx_get_key(&key_request, &cur_key);
	if (status != SGX_SUCCESS)
	{
		last_error = status;
		return false;
	}

	// save the key_id and svn's so the key can be restored even if svn's are updated
	memcpy(&file_meta_data.plain_part.meta_data_key_id, &key_request.key_id, sizeof(sgx_key_id_t)); // save this value in the meta data
	memcpy(&file_meta_data.plain_part.cpu_svn, &key_request.cpu_svn, sizeof(sgx_cpu_svn_t));
	memcpy(&file_meta_data.plain_part.isv_svn, &key_request.isv_svn, sizeof(sgx_isv_svn_t));
	return true;
#endif
}


bool protected_fs_file::restore_current_meta_data_key(const sgx_aes_gcm_128bit_key_t* import_key)
{
	if (import_key != NULL)
	{		
		memcpy(&cur_key, import_key, sizeof(sgx_aes_gcm_128bit_key_t));
		return true;
	}

	if (use_user_kdk_key == 1)
	{
		return generate_secure_blob_from_user_kdk(true);
	}

#ifdef NON_SGX_PROTECTED_FS
	assert(0);
	return false;
#else
	sgx_key_id_t empty_key_id = {0};
	if (consttime_memequal(&file_meta_data.plain_part.meta_data_key_id, &empty_key_id, sizeof(sgx_key_id_t)) == 1)
	{
		last_error = SGX_ERROR_FILE_NO_KEY_ID;
		return false;
	}

	sgx_key_request_t key_request;
	memset(&key_request, 0, sizeof(sgx_key_request_t));

	key_request.key_name = SGX_KEYSELECT_SEAL;
	key_request.key_policy = SGX_KEYPOLICY_MRSIGNER;

	key_request.attribute_mask.flags = TSEAL_DEFAULT_FLAGSMASK;
    key_request.attribute_mask.xfrm = 0x0;

	key_request.misc_mask = TSEAL_DEFAULT_MISCMASK;

	memcpy(&key_request.cpu_svn, &file_meta_data.plain_part.cpu_svn, sizeof(sgx_cpu_svn_t));
	memcpy(&key_request.isv_svn, &file_meta_data.plain_part.isv_svn, sizeof(sgx_isv_svn_t));
	memcpy(&key_request.key_id, &file_meta_data.plain_part.meta_data_key_id, sizeof(sgx_key_id_t));

	sgx_status_t status = sgx_get_key(&key_request, &cur_key);
	if (status != SGX_SUCCESS)
	{
		last_error = status;
		return false;
	}

	return true;
#endif
}


