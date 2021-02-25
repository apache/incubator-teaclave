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

theory I_FDP_ACF
  imports Main "../FDP_ACF" I_ResrcAttr 
begin

definition task_access_function::"SubjAttr\<Rightarrow>ObjAttr\<Rightarrow>bool" where
"task_access_function sattr oattr\<equiv>
(if rel_subset_usr(subj_participants sattr) (obj_owners oattr)\<and>
    info_type(obj_resrcattr oattr)=Func
 then
 True
 else False)"

definition user_access_function::"UsrAttr\<Rightarrow>ObjAttr\<Rightarrow>bool" where
"user_access_function uattr oattr\<equiv>
(if find_usrid(obj_owners oattr) uattr\<and>
 info_type(obj_resrcattr oattr)=Func
 then
 True
 else False)"

definition task_access_data::"SubjAttr\<Rightarrow>ObjAttr\<Rightarrow>bool" where
"task_access_data sattr oattr\<equiv>
(if rel_subset_usr(subj_participants sattr) (obj_owners oattr)\<and>
    info_type(obj_resrcattr oattr)=Data
 then
 True
 else False)"

definition user_access_data::"UsrAttr\<Rightarrow>ObjAttr\<Rightarrow>bool" where
"user_access_data uattr oattr\<equiv>
(if find_usrid(obj_owners oattr) uattr\<and>
 info_type(obj_resrcattr oattr)=Data
 then
 True
 else False)"

interpretation FdpAcf1 : FdpAcf1 noresrcid valid_resrcid noinfoid valid_infoid InSgx OutSgx
                                 is_insgx Device Normal is_normal Data Func is_data
                                 noresrcattr resrc_attr presrc_id info_id
                                 trust_level presrc_type info_type
                                 nousrid valid_usrid nousrattr usr_attr usrattr_id nousrattrconf
                                 usrattr_conf is_usrattrconf find_usrid delete_usrattr
                                 get_usrattr valid_usrattrconf nosubjattr subj_attr
                                 subj_callerattr subj_participants
                                 subj_resrcattr noobjattr obj_attr
                                 obj_owners obj_resrcattr rel_subset_usr
                                 task_access_function user_access_function task_access_data
                                 user_access_data
proof
  fix sattr
  fix oattr
  show "rel_subset_usr (subj_participants sattr)
       (obj_owners oattr) \<and>
       info_type (obj_resrcattr oattr) = InfoType.Func \<Longrightarrow>
       task_access_function sattr oattr" by (auto simp:task_access_function_def)
next
  fix sattr
  fix oattr
  show " \<not> rel_subset_usr (subj_participants sattr)
       (obj_owners oattr) \<and>
       info_type (obj_resrcattr oattr) = InfoType.Func \<Longrightarrow>
       \<not> task_access_function sattr oattr" by (auto simp:task_access_function_def)
next
  fix sattr
  fix oattr
  show "rel_subset_usr (subj_participants sattr)
       (obj_owners oattr) \<and>
       info_type (obj_resrcattr oattr) \<noteq> InfoType.Func \<Longrightarrow>
       \<not>task_access_function sattr oattr" by (auto simp:task_access_function_def)
next
  fix oattr
  fix uattr
  show "find_usrid (obj_owners oattr) uattr \<and>
       info_type (obj_resrcattr oattr) = InfoType.Func \<Longrightarrow>
       user_access_function uattr oattr" by (auto simp:user_access_function_def)
next
  fix oattr
  fix uattr
  show "\<not> find_usrid (obj_owners oattr) uattr \<and>
       info_type (obj_resrcattr oattr) = InfoType.Func \<Longrightarrow>
       \<not> user_access_function uattr oattr" by (auto simp:user_access_function_def)
next
  fix oattr
  fix uattr
  show "find_usrid (obj_owners oattr) uattr \<and>
       info_type (obj_resrcattr oattr) \<noteq> InfoType.Func \<Longrightarrow>
       \<not> user_access_function uattr oattr" by (auto simp:user_access_function_def)
next
  fix sattr
  fix oattr
  show "rel_subset_usr (subj_participants sattr)
       (obj_owners oattr) \<and>
       info_type (obj_resrcattr oattr) = Data \<Longrightarrow>
       task_access_data sattr oattr" by (auto simp:task_access_data_def)
next
  fix sattr
  fix oattr
  show "\<not> rel_subset_usr (subj_participants sattr)
       (obj_owners oattr) \<and>
       info_type (obj_resrcattr oattr) = Data \<Longrightarrow>
       \<not> task_access_data sattr oattr" by (auto simp:task_access_data_def)
next
  fix sattr
  fix oattr
  show "rel_subset_usr (subj_participants sattr)
       (obj_owners oattr) \<and>
       info_type (obj_resrcattr oattr) \<noteq> Data \<Longrightarrow>
       \<not>task_access_data sattr oattr" by (auto simp:task_access_data_def)
next
  fix oattr
  fix uattr
  show "find_usrid (obj_owners oattr) uattr \<and>
       info_type (obj_resrcattr oattr) = Data \<Longrightarrow>
       user_access_data uattr oattr" by (auto simp:user_access_data_def)
next
  fix oattr
  fix uattr
  show "\<not> find_usrid (obj_owners oattr) uattr \<and>
       info_type (obj_resrcattr oattr) = Data \<Longrightarrow>
       \<not> user_access_data uattr oattr" by (auto simp:user_access_data_def)
next
  fix oattr
  fix uattr
  show "find_usrid (obj_owners oattr) uattr \<and>
       info_type (obj_resrcattr oattr) \<noteq> Data \<Longrightarrow>
       \<not> user_access_data uattr oattr" by (auto simp:user_access_data_def)
next
  fix oattr
  fix uattr
  show "\<not> (find_usrid (obj_owners oattr) uattr \<and>
       info_type (obj_resrcattr oattr) = Data) \<Longrightarrow>
       \<not> user_access_data uattr oattr" by (auto simp:user_access_data_def)
next
  fix sattr
  fix oattr
  show "\<not> (rel_subset_usr (subj_participants sattr) (obj_owners oattr) \<and>
       info_type (obj_resrcattr oattr) = Data) \<Longrightarrow>
       \<not> task_access_data sattr oattr" by (auto simp:task_access_data_def)
qed

end
