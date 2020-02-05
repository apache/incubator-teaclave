#pragma once

#ifndef _NON_SGX_PROTECTED_FS_H_
#define _NON_SGX_PROTECTED_FS_H_

#include "sgx_error.h"

#include <stdint.h>
#include <string.h>

#define SGX_AESGCM_IV_SIZE              12
#define SGX_AESGCM_KEY_SIZE             16
#define SGX_AESGCM_MAC_SIZE             16
#define SGX_CMAC_KEY_SIZE               16
#define SGX_CMAC_MAC_SIZE               16

typedef uint8_t aead_128bit_key_t[SGX_AESGCM_KEY_SIZE];
typedef uint8_t aead_128bit_tag_t[SGX_AESGCM_MAC_SIZE];
typedef uint8_t cmac_128bit_key_t[SGX_CMAC_KEY_SIZE];
typedef uint8_t cmac_128bit_tag_t[SGX_CMAC_MAC_SIZE];
typedef uint8_t sgx_key_128bit_t[16];

typedef aead_128bit_key_t sgx_aes_gcm_128bit_key_t;
typedef aead_128bit_tag_t sgx_aes_gcm_128bit_tag_t;
typedef cmac_128bit_tag_t sgx_cmac_128bit_key_t;
typedef cmac_128bit_tag_t sgx_cmac_128bit_tag_t;

#include <pthread.h>
#define sgx_thread_mutex_lock pthread_mutex_lock
#define sgx_thread_mutex_unlock pthread_mutex_unlock
#define sgx_thread_mutex_init pthread_mutex_init
#define sgx_thread_mutex_t pthread_mutex_t
#define sgx_thread_mutex_destroy pthread_mutex_destroy

//defined in sgx_key.h
#define SGX_KEYID_SIZE    32
#define SGX_CPUSVN_SIZE   16
typedef uint16_t                   sgx_isv_svn_t;

typedef struct _sgx_key_id_t
{
    uint8_t                        id[SGX_KEYID_SIZE];
} sgx_key_id_t;

typedef struct _sgx_cpu_svn_t
{
    uint8_t                        svn[SGX_CPUSVN_SIZE];
} sgx_cpu_svn_t;

// sgx_attributes.h
typedef struct _attributes_t
{
    uint64_t      flags;
    uint64_t      xfrm;
} sgx_attributes_t;



sgx_status_t sgx_rijndael128_cmac_msg(const sgx_cmac_128bit_key_t *p_key, const uint8_t *p_src,
                                      uint32_t src_len, sgx_cmac_128bit_tag_t *p_mac);

sgx_status_t sgx_rijndael128GCM_encrypt(const sgx_aes_gcm_128bit_key_t *p_key, const uint8_t *p_src, uint32_t src_len,
                                        uint8_t *p_dst, const uint8_t *p_iv, uint32_t iv_len, const uint8_t *p_aad, uint32_t aad_len,
                                        sgx_aes_gcm_128bit_tag_t *p_out_mac);

sgx_status_t sgx_rijndael128GCM_decrypt(const sgx_aes_gcm_128bit_key_t *p_key, const uint8_t *p_src,
                                        uint32_t src_len, uint8_t *p_dst, const uint8_t *p_iv, uint32_t iv_len,
                                        const uint8_t *p_aad, uint32_t aad_len, const sgx_aes_gcm_128bit_tag_t *p_in_mac);

sgx_status_t read_rand(uint8_t *buf, size_t size);
#define sgx_read_rand read_rand

int memset_s(void *s, size_t smax, int c, size_t n);
int consttime_memequal(const void *b1, const void *b2, size_t len);

#endif // _NON_SGX_PROTECTED_FS_H_