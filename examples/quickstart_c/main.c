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

#include <arpa/inet.h>
#include <assert.h>
#include <mesatee/mesatee.h>
#include <netinet/in.h>
#include <stdio.h>

void single_party_task(mesatee_enclave_info_t *enclave_info) {
  struct sockaddr_in tms_addr;
  struct sockaddr_in tdfs_addr;
  char recvbuf[128] = {0};
  int ret;

  tms_addr.sin_family = AF_INET;
  tms_addr.sin_addr.s_addr = inet_addr("127.0.0.1");
  tms_addr.sin_port = htons(5554);

  tdfs_addr.sin_family = AF_INET;
  tdfs_addr.sin_addr.s_addr = inet_addr("127.0.0.1");
  tdfs_addr.sin_port = htons(5065);

  printf("[+] This is a single-party task: echo\n");
  
  mesatee_t *context = mesatee_context_new(enclave_info, "uid1", "token1",
                                           (struct sockaddr *)&tms_addr,
                                           (struct sockaddr *)&tdfs_addr);
  assert(context != NULL);

  mesatee_task_t *task = mesatee_create_task(context, "echo");
  assert(task != NULL);

  ret = mesatee_task_invoke_with_payload(task, "haha", 4, recvbuf, 128);
  assert(ret > 0);

  printf("Response: %s\n", recvbuf);

  mesatee_task_free(task);
  mesatee_context_free(context);
}

int main() {
  mesatee_init();

  mesatee_auditor_set_t *auditors = mesatee_auditor_set_new();
  mesatee_auditor_set_add_auditor(auditors,
                                  "../services/auditors/godzilla/godzilla.public.der",
                                  "../services/auditors/godzilla/godzilla.sign.sha256");
  mesatee_auditor_set_add_auditor(
      auditors, "../services/auditors/optimus_prime/optimus_prime.public.der",
      "../services/auditors/optimus_prime/optimus_prime.sign.sha256");
  mesatee_auditor_set_add_auditor(
      auditors, "../services/auditors/albus_dumbledore/albus_dumbledore.public.der",
      "../services/auditors/albus_dumbledore/albus_dumbledore.sign.sha256");

  assert(auditors != NULL);

  mesatee_enclave_info_t *enclave_info =
      mesatee_enclave_info_load(auditors, "../services/enclave_info.toml");

  assert(enclave_info != NULL);

  single_party_task(enclave_info);

  mesatee_enclave_info_free(enclave_info);
  mesatee_auditor_set_free(auditors);
  return 0;
}
