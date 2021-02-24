(*
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
 *)

theory I_SysId
  imports Main "../SysId"
begin

typedef ResrcId = "{x. x \<ge> (0::nat) \<and> x < 100}"
proof
  show "(0::nat) \<in> {x. 0 \<le> x \<and> x < 100}"
  proof
    show"(0::nat) \<le> 0 \<and> 0 < (100::nat)"
    proof
      show"(0::nat) \<le> 0" by auto
    next
      show"(0::nat) < 100" by auto
    qed
  qed
qed

definition noresrcid::ResrcId where "noresrcid\<equiv>Abs_ResrcId 0"

definition valid_resrcid::"ResrcId\<Rightarrow>bool" where
"valid_resrcid rid\<equiv>(rid\<noteq>noresrcid)"

lemma RESRCIDLLR1:"rid\<noteq>noresrcid\<Longrightarrow>valid_resrcid rid"
proof -
  assume "rid\<noteq>noresrcid"
  from this show "valid_resrcid rid" by(auto simp:valid_resrcid_def)
qed

lemma RESRCIDLLR2:"rid=noresrcid\<Longrightarrow>\<not>valid_resrcid rid"
proof
  assume 0:"rid = noresrcid"
  assume 1:"valid_resrcid rid"
  from 1 have 2:"rid\<noteq>noresrcid" by(auto simp:valid_resrcid_def)
  from 0 2 show False by auto
qed

typedef UsrId = "{x. x \<ge> (100::nat) \<and> x < 200}"
proof
  show "(100::nat) \<in> {x. 100 \<le> x \<and> x < 200}"
  proof
    show"(100::nat) \<le> 100 \<and> 100 < (200::nat)"
    proof
      show"(100::nat) \<le> 100" by auto
    next
      show"(100::nat) < 200" by auto
    qed
  qed
qed

definition nousrid::UsrId where "nousrid\<equiv>Abs_UsrId 100"

definition valid_usrid::"UsrId\<Rightarrow>bool" where
"valid_usrid uid\<equiv>(uid\<noteq>nousrid)"

lemma USRIDLLR1:"uid\<noteq>nousrid\<Longrightarrow>valid_usrid uid"
proof -
  assume "uid\<noteq>nousrid"
  from this show "valid_usrid uid" by(auto simp:valid_usrid_def)
qed

lemma USRIDLLR2:"uid=nousrid\<Longrightarrow>\<not>valid_usrid uid"
proof
  assume 0:"uid = nousrid"
  assume 1:"valid_usrid uid"
  from 1 have 2:"uid\<noteq>nousrid" by(auto simp:valid_usrid_def)
  from 0 2 show False by auto
qed 

typedef InfoId = "{x. x \<ge> (200::nat) \<and> x < 300}"
proof
  show "(200::nat) \<in> {x. 200 \<le> x \<and> x < 300}"
  proof
    show"(200::nat) \<le> 200 \<and> 200 < (300::nat)"
    proof
      show"(200::nat) \<le> 200" by auto
    next
      show"(200::nat) < 300" by auto
    qed
  qed
qed

definition noinfoid::InfoId where "noinfoid\<equiv>Abs_InfoId 200"
  
definition valid_infoid::"InfoId\<Rightarrow>bool" where
"valid_infoid iid\<equiv>(iid\<noteq>noinfoid)"

lemma INFOIDLLR1:"iid\<noteq>noinfoid\<Longrightarrow>valid_infoid iid"
proof -
  assume "iid\<noteq>noinfoid"
  from this show "valid_infoid iid" by(auto simp:valid_infoid_def)
qed

lemma INFOIDLLR2:"iid=noinfoid\<Longrightarrow>\<not>valid_infoid iid"
proof
  assume 0:"iid = noinfoid"
  assume 1:"valid_infoid iid"
  from 1 have 2:"iid\<noteq>noinfoid" by(auto simp:valid_infoid_def)
  from 0 2 show False by auto
qed 

interpretation SysId_ResrcId : SysId noresrcid valid_resrcid
proof
  fix x
  assume "x\<noteq>noresrcid"
  from this show "valid_resrcid x" by(auto simp:RESRCIDLLR1)
next
  fix x
  assume "x=noresrcid"
  from this show "\<not>valid_resrcid x" by(auto simp:RESRCIDLLR2)
qed

interpretation SysId_UsrId : SysId nousrid valid_usrid
proof
  fix x
  assume "x\<noteq>nousrid"
  from this show "valid_usrid x" by(auto simp:USRIDLLR1)
next
  fix x
  assume "x = nousrid"
  from this show " \<not>valid_usrid x" by(auto simp:USRIDLLR2)
qed

interpretation SysId_InfoId : SysId noinfoid valid_infoid
proof
  fix x
  assume "x\<noteq>noinfoid"
  from this show "valid_infoid x" by(auto simp:INFOIDLLR1)
next
  fix x
  assume "x = noinfoid"
  from this show "\<not>valid_infoid x" by(auto simp:INFOIDLLR2)
qed

end
