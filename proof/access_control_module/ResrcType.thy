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

theory ResrcType
  imports Main SysId TrustLevel
begin

locale PresrcType=
  fixes device::"'presrctype"
    and normal::"'presrctype"
    and is_normal::"'presrctype\<Rightarrow>bool"
  assumes PRESRCTYPEHLR1:"\<not>is_normal device"
  assumes PRESRCTYPEHLR2:"is_normal normal"
  assumes PRESRCTYPEHLR3:"(x::'presrctype)=device\<or>x=normal"
begin

lemma PRESRCTYPEHLR4:"device\<noteq>normal"
proof
  assume 0:"device=normal"
  from PRESRCTYPEHLR1 have "\<not>is_normal normal" by(auto simp: 0)
  from this show "False" by(auto simp: PRESRCTYPEHLR2)
qed

end

print_locale! PresrcType

locale TresrcType=tid:SysId notid valid_tid
    for notid::"'tid"
    and valid_tid::"'tid\<Rightarrow>bool"+
  fixes tresrc_type::"'tid\<Rightarrow>'tid\<Rightarrow>'tresrctype"
    and core_id::"'tresrctype\<Rightarrow>'tid"
    and period_id::"'tresrctype\<Rightarrow>'tid"
  assumes TRESRCTYPEHLR1:"core_id(tresrc_type cid pid)=cid"
  assumes TRESRCTYPEHLR2:"period_id(tresrc_type cid pid)=pid"
  assumes TRESRCTYPEHLR3:"\<exists>y::'tid. \<exists>z::'tid. x=tresrc_type y z"



print_locale! TresrcType

end