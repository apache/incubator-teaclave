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

theory I_ResrcType
  imports Main "../ResrcType" I_SysId
begin

datatype PresrcType = is_normal:Normal | Device

print_theorems

interpretation PresrcType : PresrcType  Device Normal is_normal
proof
  show "\<not> is_normal Device" by auto
next
  show "is_normal Normal" by auto
next
  fix x
  show "x = Device \<or> x = Normal"
  proof (cases x)
    assume "x=Device"
    from this show "x = Device \<or> x = Normal" by auto
  next
    assume "x=Normal"
    from this show "x = Device \<or> x = Normal" by auto
  qed
qed

datatype TresrcType=tresrc_type (core_id:ResrcId) (period_id:ResrcId)


interpretation TresrcType : TresrcType 
               noresrcid valid_resrcid tresrc_type core_id period_id
proof
  fix cid pid
  show "core_id (tresrc_type cid pid) = cid" by auto
next
  fix cid pid
  show "period_id (tresrc_type cid pid) = pid" by auto
next
  fix x
  show "\<exists>y z. x = tresrc_type y z"
  proof (cases x)
    fix x1 x2
    assume "x = tresrc_type x1 x2"
    from this show " \<exists>y z. x = tresrc_type y z" by auto
  qed
qed




end