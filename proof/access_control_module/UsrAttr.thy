(*
# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.
*)

theory UsrAttr
  imports Main SysId AttrConf
begin

locale UsrAttr=SysId noid valid_id
  for noid::'id
    and valid_id::"'id\<Rightarrow>bool" +
  fixes nousrattr::'usrattr
    and usr_attr::"'id\<Rightarrow>'usrattr"
    and usrattr_id::"'usrattr\<Rightarrow>'id"
  assumes USRATTRHLR1:"usrattr_id nousrattr=noid"
  assumes USRATTRHLR2:"usrattr_id(usr_attr uid)=uid"
  assumes USRATTRHLR3:"\<exists>uid. x=usr_attr uid\<or>x=nousrattr"

print_locale! UsrAttr

locale UsrAttrConf=UsrAttr noid valid_id nousrattr usr_attr usrattr_id +
                   AttrConf noid nousrattr nousrattrconf usrattr_conf is_usrattrconf usrattr_id
                            find_usrid delete_usrattr get_usrattr valid_usrattrconf
  for noid::'id
    and valid_id::"'id\<Rightarrow>bool"
    and nousrattr::'usrattr
    and usr_attr::"'id\<Rightarrow>'usrattr"
    and usrattr_id::"'usrattr\<Rightarrow>'id"
    and nousrattrconf::"'usrattrconf"
    and usrattr_conf::"'usrattrconf\<Rightarrow>'usrattr\<Rightarrow>'usrattrconf"
    and is_usrattrconf::"'usrattrconf\<Rightarrow>bool"
    and find_usrid::"'usrattrconf\<Rightarrow>'usrattr\<Rightarrow>bool" 
    and delete_usrattr::"'usrattrconf\<Rightarrow>'usrattr\<Rightarrow>'usrattrconf"
    and get_usrattr::"'usrattrconf\<Rightarrow>'id\<Rightarrow>'usrattr"
    and valid_usrattrconf::"'usrattrconf\<Rightarrow>bool"


print_locale! UsrAttrConf


end