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

/**
 * Open a protected file as input
 * 
 * @param fid the uid of the file, c string pointer
 * 
 * @return file handler, -1 if error occurs
 */
int teaclave_open_input(char *file_id);

/**
 * Create/Open? a protected file as output
 * 
 * @param fid the uid of the file, c string pointer
 * 
 * @return file handler, -1 if error occurs
 */
int teaclave_create_output(char *file_id);

/**
 * Read content from a file to a buffer
 * 
 * @param fd file handler returned by `teaclave_open_input`
 * @param out_buf the pointer to output buffer
 * @param buf_size the total size in bytes of the output buffer
 * 
 * @return bytes read from the file, -1 if error occurs
 */
int teaclave_read_file(int fd, void *out_buf, int buf_size);

/**
 * Write content from a buffer to a file
 * 
 * @param fd file handler returned by `teaclave_create_output`
 * @param buf the pointer to the buffer holding content to write
 * @param buf_size the total size in bytes to read from the buffer and write to the file
 * 
 * @return bytes written to the file, -1 if error occurs
 */
int teaclave_write_file(int fd, void *buf, int buf_size);

/**
 * Close a file handler
 * 
 * @param fd file handler to close
 * 
 * @return 0 if succeed, -1 otherwise
 */
int teaclave_close_file(int fd);
