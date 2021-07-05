// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#include "protected_fs_config.h"

#ifdef NON_SGX_PROTECTED_FS

#include "non_sgx_protected_fs.h"
#include "openssl/cmac.h"
#include "openssl/err.h"

#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>

/* Message Authentication - Rijndael 128 CMAC
* Parameters:
*   Return: sgx_status_t  - SGX_SUCCESS or failure as defined sgx_error.h
*   Inputs: sgx_cmac_128bit_key_t *p_key - Pointer to key used in encryption/decryption operation
*           uint8_t *p_src - Pointer to input stream to be MACed
*           uint32_t src_len - Length of input stream to be MACed
*   Output: sgx_cmac_gcm_128bit_tag_t *p_mac - Pointer to resultant MAC */
sgx_status_t sgx_rijndael128_cmac_msg(const sgx_cmac_128bit_key_t *p_key, const uint8_t *p_src,
                                      uint32_t src_len, sgx_cmac_128bit_tag_t *p_mac)
{
	void* pState = NULL;

	if ((p_key == NULL) || (p_src == NULL) || (p_mac == NULL))  {
		return SGX_ERROR_INVALID_PARAMETER;
	}

	size_t mactlen;
	sgx_status_t ret = SGX_ERROR_UNEXPECTED;

	do {
		//create a new ctx of CMAC
		//
		pState = CMAC_CTX_new();
		if (pState == NULL) {
			ret = SGX_ERROR_OUT_OF_MEMORY;
			break;
		}

		// init CMAC ctx with the corresponding size, key and AES alg.
		//
		if (!CMAC_Init((CMAC_CTX*)pState, (const void *)p_key, SGX_CMAC_KEY_SIZE, EVP_aes_128_cbc(), NULL)) {
			break;
		}

		// perform CMAC hash on p_src
		//
		if (!CMAC_Update((CMAC_CTX *)pState, p_src, src_len)) {
			break;
		}

		// finalize CMAC hashing
		//
		if (!CMAC_Final((CMAC_CTX*)pState, (unsigned char*)p_mac, &mactlen)) {
			break;
		}

		//validate mac size
		//
		if (mactlen != SGX_CMAC_MAC_SIZE) {
			break;
		}

		ret = SGX_SUCCESS;
	} while (0);

	// we're done, clear and free CMAC ctx
	//
	if (pState) {
		CMAC_CTX_free((CMAC_CTX*)pState);
	}
	return ret;
}



#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
sgx_status_t read_rand(uint8_t *buf, size_t size)
{
    int fd = open("/dev/urandom", O_RDONLY);
    if (fd == -1) {
        return SGX_ERROR_UNEXPECTED; 
    }
    ssize_t ret = read(fd, buf, size);
    if (ret == -1) {
        return SGX_ERROR_UNEXPECTED; 
    }
    ret = close(fd);
    if (ret == -1) {
        return SGX_ERROR_UNEXPECTED; 
    }
    return SGX_SUCCESS;
}

int memset_s(void *s, size_t smax, int c, size_t n)
{
    int err = 0;

    if (s == NULL) {
        err = EINVAL;
        goto out;
    }

    if (n > smax) {
        err = EOVERFLOW;
        n = smax;
    }

    /* Calling through a volatile pointer should never be optimised away. */
    memset(s, c, n);

    out:
    if (err == 0)
        return 0;
    else {
        errno = err;
        /* XXX call runtime-constraint handler */
        return err;
    }
}


sgx_status_t sgx_rijndael128GCM_encrypt(const sgx_aes_gcm_128bit_key_t *p_key, const uint8_t *p_src, uint32_t src_len,
                                        uint8_t *p_dst, const uint8_t *p_iv, uint32_t iv_len, const uint8_t *p_aad, uint32_t aad_len,
                                        sgx_aes_gcm_128bit_tag_t *p_out_mac)
{
    if ((src_len >= INT_MAX) || (aad_len >= INT_MAX) || (p_key == NULL) || ((src_len > 0) && (p_dst == NULL)) || ((src_len > 0) && (p_src == NULL))
        || (p_out_mac == NULL) || (iv_len != SGX_AESGCM_IV_SIZE) || ((aad_len > 0) && (p_aad == NULL))
        || (p_iv == NULL) || ((p_src == NULL) && (p_aad == NULL)))
    {
        return SGX_ERROR_INVALID_PARAMETER;
    }
    sgx_status_t ret = SGX_ERROR_UNEXPECTED;
    int len = 0;
    EVP_CIPHER_CTX * pState = NULL;


    do {
        // Create and init ctx
        //
        if (!(pState = EVP_CIPHER_CTX_new()))
        {
            ret = SGX_ERROR_OUT_OF_MEMORY;
            break;
        }

        // Initialise encrypt, key and IV
        //
        if (1 != EVP_EncryptInit_ex(pState, EVP_aes_128_gcm(), NULL, (unsigned char*)p_key, p_iv))
        {
            break;
        }

        // Provide AAD data if exist
        //
        if (aad_len > 0)
        {
            if (1 != EVP_EncryptUpdate(pState, NULL, &len, p_aad, aad_len))
            {
                break;
            }
        }

        // Provide the message to be encrypted, and obtain the encrypted output.
        //
        if(src_len > 0)
        {
            if (1 != EVP_EncryptUpdate(pState, p_dst, &len, p_src, src_len))
            {
                break;
            }
        }
        // Finalise the encryption
        //
        if (1 != EVP_EncryptFinal_ex(pState, p_dst + len, &len))
        {
            break;
        }

        // Get tag
        //
        if (1 != EVP_CIPHER_CTX_ctrl(pState, EVP_CTRL_GCM_GET_TAG, SGX_AESGCM_MAC_SIZE, p_out_mac))
        {
            break;
        }
        ret = SGX_SUCCESS;
    } while (0);


    // Clean up and return
    //
    if (pState) {
            EVP_CIPHER_CTX_free(pState);
    }
    return ret;
}

sgx_status_t sgx_rijndael128GCM_decrypt(const sgx_aes_gcm_128bit_key_t *p_key, const uint8_t *p_src,
                                        uint32_t src_len, uint8_t *p_dst, const uint8_t *p_iv, uint32_t iv_len,
                                        const uint8_t *p_aad, uint32_t aad_len, const sgx_aes_gcm_128bit_tag_t *p_in_mac)
{
    uint8_t l_tag[SGX_AESGCM_MAC_SIZE] = {0};

    if ((src_len >= INT_MAX) || (aad_len >= INT_MAX) || (p_key == NULL) || ((src_len > 0) && (p_dst == NULL)) || ((src_len > 0) && (p_src == NULL))
        || (p_in_mac == NULL) || (iv_len != SGX_AESGCM_IV_SIZE) || ((aad_len > 0) && (p_aad == NULL))
        || (p_iv == NULL) || ((p_src == NULL) && (p_aad == NULL)))
    {
        return SGX_ERROR_INVALID_PARAMETER;
    }
    int len = 0;
    sgx_status_t ret = SGX_ERROR_UNEXPECTED;
    EVP_CIPHER_CTX * pState = NULL;

    // Autenthication Tag returned by Decrypt to be compared with Tag created during seal
    //
    memset_s(&l_tag, SGX_AESGCM_MAC_SIZE, 0, SGX_AESGCM_MAC_SIZE);
    memcpy(l_tag, p_in_mac, SGX_AESGCM_MAC_SIZE);

    do {
        // Create and initialise the context
        //
        if (!(pState = EVP_CIPHER_CTX_new()))
        {
            ret = SGX_ERROR_OUT_OF_MEMORY;
            break;
        }

        // Initialise decrypt, key and IV
        //
        if (!EVP_DecryptInit_ex(pState, EVP_aes_128_gcm(), NULL, (unsigned char*)p_key, p_iv))
        {
            break;
        }

        // Provide AAD data if exist
        //
        if (aad_len > 0) 
        {
            if (!EVP_DecryptUpdate(pState, NULL, &len, p_aad, aad_len)) 
            {
                break;
            }
        }

        // Decrypt message, obtain the plaintext output
        //
        if(src_len > 0)
        {       
            if (!EVP_DecryptUpdate(pState, p_dst, &len, p_src, src_len))
            {
                break;
            }
        }

        // Update expected tag value
        //
        if (!EVP_CIPHER_CTX_ctrl(pState, EVP_CTRL_GCM_SET_TAG, SGX_AESGCM_MAC_SIZE, l_tag))
        {
            break;
        }

        // Finalise the decryption. A positive return value indicates success,
        // anything else is a failure - the plaintext is not trustworthy.
        //
        if (EVP_DecryptFinal_ex(pState, p_dst + len, &len) <= 0)
        {
            break;
        }
        ret = SGX_SUCCESS;
    } while (0);

    // Clean up and return
    //
    if (pState != NULL)
    {
        EVP_CIPHER_CTX_free(pState);
    }
    memset_s(&l_tag, SGX_AESGCM_MAC_SIZE, 0, SGX_AESGCM_MAC_SIZE);
    return ret;
}

int
consttime_memequal(const void *b1, const void *b2, size_t len)
{
	const unsigned char *c1 = (const unsigned char*)b1, *c2 = (const unsigned char*)b2;
	unsigned int res = 0;

	while (len--)
		res |= *c1++ ^ *c2++;

	/*
	 * Map 0 to 1 and [1, 256) to 0 using only constant-time
	 * arithmetic.
	 *
	 * This is not simply `!res' because although many CPUs support
	 * branchless conditional moves and many compilers will take
	 * advantage of them, certain compilers generate branches on
	 * certain CPUs for `!res'.
	 */
	return (1 & ((res - 1) >> 8));
}

#endif
