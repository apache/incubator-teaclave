/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 *
 */

#include "teaclave_context.h"

int strlen(const char *str)
{
    const char *s;

    for (s = str; *s; ++s)
        ;
    return (s - str);
}

int atoi(const char *str)
{
    int result = 0;
    int sign = 0;

    while (*str == ' ' || *str == '\t' || *str == '\n')
        ++str;

    if (*str == '-')
    {
        sign = 1;
        ++str;
    }
    else if (*str == '+')
    {
        ++str;
    }

    // proc numbers
    while (*str >= '0' && *str <= '9')
    {
        result = result * 10 + *str - '0';
        ++str;
    }

    if (sign == 1)
        return -result;
    else
        return result;
}

int entrypoint(int argc, char *argv[])
{
    if (argc < 6)
    {
        return -1;
    }

    if ((argv[0] == 0) || argv[2] == 0 || argv[4] == 0)
    {
        return -1;
    }

    char *input_fid_a = argv[1];
    char *input_fid_b = argv[3];
    char *output_fid = argv[5];

    int buf_len = 11, rv = -2;
    char buf_a[buf_len];
    char buf_b[buf_len];

    int input_handle_a = teaclave_open_input(input_fid_a);
    int input_handle_b = teaclave_open_input(input_fid_b);

    int output_handle = teaclave_create_output(output_fid);

    // check failure
    if ((input_handle_a == -1) || (input_handle_b == -1) || (output_handle == -1))
    {
        return -1;
    }

    int read_bytes_a = teaclave_read_file(input_handle_a, buf_a, buf_len - 1);
    int read_bytes_b = teaclave_read_file(input_handle_b, buf_b, buf_len - 1);

    if ((read_bytes_a == -1) || (read_bytes_b == -1))
    {
        return -1;
    }

    int a = atoi(buf_a), b = atoi(buf_b);

    if (a > b)
    {
        rv = teaclave_write_file(output_handle, input_fid_a, strlen(input_fid_a));
    }
    else
    {
        rv = teaclave_write_file(output_handle, input_fid_b, strlen(input_fid_b));
    }

    teaclave_close_file(input_handle_a);
    teaclave_close_file(input_handle_b);
    teaclave_close_file(output_handle);

    return rv;
}
