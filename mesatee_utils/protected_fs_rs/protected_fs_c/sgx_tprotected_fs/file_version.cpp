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

#include "se_version.h"

#define __CONCAT(x, y) x/**/y

#define SGX_TPROTECTEDFS_VERSION_STR  __CONCAT("SGX_TPROTECTEDFS_VERSION_", STRFILEVER)

#ifdef __cplusplus
extern "C" {
#endif
__attribute__((visibility("default")))
char sgx_tprotectedfs_version[] = SGX_TPROTECTEDFS_VERSION_STR;
#ifdef __cplusplus
}
#endif
