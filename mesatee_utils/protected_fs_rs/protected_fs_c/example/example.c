#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "sgx_tprotected_fs.h"

int main() {
    const char *file_name = "data_file";
    sgx_key_128bit_t key = {'0', '1', '2', '3', '4', '5', '6', '7',
                            '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'};
    SGX_FILE* fd = sgx_fopen(file_name, "w", &key);
    if (fd == NULL) {
        fprintf(stderr, "create file failed");
        return -1;
    }

    int unit_size = 0x10000;
    unsigned char *buffer = (unsigned char *)malloc(unit_size);
    if (buffer == NULL) {
        fprintf(stderr, "malloc buffer failed");
        sgx_fclose(fd);
        return -1;
    }
    
    memset(buffer, 0x90, unit_size);
    
    
    for (int i = 0; i < unit_size; i++) {
        size_t n = sgx_fwrite(buffer, 1, unit_size, fd);
        if (n != unit_size) {
            fprintf(stderr, "write file failed: 0x%lx, unit_size: 0x%x, i: 0x%x\n", n, unit_size, i);
            sgx_fclose(fd);
            return -1;
        }
    }

    sgx_fclose(fd);

    return 0;
}
