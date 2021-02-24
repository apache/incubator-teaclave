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

theory I_UsrAttr
  imports Main "../UsrAttr" I_SysId 
begin

record UsrAttr=
  usrattr_id::UsrId

definition usr_attr::"UsrId\<Rightarrow>UsrAttr" where
"usr_attr uid\<equiv>\<lparr>usrattr_id=uid\<rparr>"

definition nousrattr::UsrAttr where
"nousrattr\<equiv>\<lparr>usrattr_id=nousrid\<rparr>"

interpretation UsrAttr : UsrAttr nousrid valid_usrid nousrattr usr_attr usrattr_id
proof
  show "usrattr_id nousrattr = nousrid" by (auto simp:nousrattr_def)
next
  fix uid
  show " usrattr_id (usr_attr uid) = uid" by (auto simp:usr_attr_def)
next
  fix x
  show "\<exists>uid. x = usr_attr uid \<or> x = nousrattr"
  proof
    show "x = usr_attr (usrattr_id x) \<or> x = nousrattr" by (auto simp:usr_attr_def)
  qed
qed



end