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
 */


/* DO NOT MODIFY THIS MANUALLY! This file was generated using cbindgen.
 * To generate this file:
 * 1. Get the latest cbindgen using `cargo install --force cbindgen`
 * 2. Run `rustup run nightly cbindgen ./teaclave_context -c cbindgen.toml -o teaclave_context.h`.
 */

/**
 * Close a file handler
 *
 * # Arguments
 *
 * * `fd` - file handler returned by `teaclave_open_input`
 *
 * # Return
 *
 * 0 if succeed, -1 otherwise
 */
extern int teaclave_close_file(int fd);

/**
 * Write content from a buffer to a file
 *
 * # Arguments
 *
 * * `fd` - file handler returned by `teaclave_open_input`
 * * `in_buf` - the pointer to the buffer holding content to write
 * * `buf_size` - the total size in bytes to read from the buffer and write to the file
 *
 * # Return
 *
 * bytes written to the file, -1 if error occurs
 */
extern int teaclave_write_file(int fd, char *in_buf, int buf_size);

/**
 * Read content from a file to a buffer
 *
 * # Arguments
 *
 * * `fd` - file handler returned by `teaclave_open_input`
 * * `out_buf` - the pointer to output buffer
 * * `buf_size` - the total size in bytes of the output buffer
 *
 * # Return
 *
 * bytes read from the file, -1 if error occurs
 */
extern int teaclave_read_file(int fd, char *out_buf, int buf_size);

/**
 * Create or open a protected file as output
 *
 * # Arguments
 *
 * * `fid` - the uid of the file, c string pointer
 *
 * # Return
 *
 * file handler, -1 if error occurs
 */
extern int teaclave_create_output(char *fid);

/**
 * Open a protected file as input
 *
 * # Arguments
 *
 * * `fid` - the uid of the file, c string pointer
 *
 * # Return
 *
 * file handler, -1 if error occurs
 */
extern int teaclave_open_input(char *fid);
