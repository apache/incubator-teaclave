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

#pragma once

#ifndef _LRU_CACHE_H_
#define _LRU_CACHE_H_

#include <assert.h>

#include <list>

/* STL map is implemented as a tree, hence all operations are O(logN)
   STL unordered_map is implemented as hash, hence all operations are O(1)
   http://stackoverflow.com/questions/2196995/is-there-any-advantage-of-using-map-over-unordered-map-in-case-of-trivial-keys
   http://stackoverflow.com/questions/3902644/choosing-between-stdmap-and-stdunordered-map
   */

#include <unordered_map>

/* this hasher code was needed since strict ansi don't allow 'long long' type
 * adding -U__STRICT_ANSI__ to the compilation flags solved the issue
 * leaving this code here for future reference
namespace stlpmtx_std
{
template<> struct hash<uint64_t> {
  size_t operator()(uint64_t x) const { return (size_t)x; }
  };
}
*/

typedef struct _list_node
{
	uint64_t key;
} list_node_t;

typedef struct _map_node
{
	void* data;
	std::list<list_node_t*>::iterator list_it;
} map_node_t;

typedef std::unordered_map<uint64_t, map_node_t*>::iterator map_iterator;
typedef std::list<list_node_t*>::iterator list_iterator;

class lru_cache
{
private:
	std::list<list_node_t*> list;
	std::unordered_map<uint64_t, map_node_t*> map;

	list_iterator m_it; // for get_first and get_next sequence

public:	
	lru_cache();
	~lru_cache();

	void rehash(uint32_t size_);

	bool add(uint64_t key, void* p);
	void* get(uint64_t key);
	void* find(uint64_t key); // only returns the object, do not bump it to the head
	uint32_t size();

	void* get_first();
	void* get_next();
	void* get_last();
	void remove_last();
};

#endif // _LRU_CACHE_H_
