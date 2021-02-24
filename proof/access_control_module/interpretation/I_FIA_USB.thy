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

theory I_FIA_USB
  imports Main "../FIA_USB" I_ResrcAttr 
begin


definition user_bind_subject::"UsrAttr\<Rightarrow>SubjAttr\<Rightarrow>SubjAttr" where
"user_bind_subject uattr sattr\<equiv>
(if find_usrid(subj_participants sattr) uattr
 then 
 subj_attr uattr (subj_participants sattr) (subj_resrcattr sattr)
 else
 nosubjattr)"

interpretation FiaUsb1 : FiaUsb1 nousrid valid_usrid noresrcid valid_resrcid noinfoid valid_infoid
                                 InSgx OutSgx is_insgx Device Normal is_normal Data
                                 Func is_data noresrcattr resrc_attr presrc_id
                                 info_id trust_level presrc_type info_type nousrattr 
                                 usr_attr usrattr_id nousrattrconf usrattr_conf is_usrattrconf 
                                 find_usrid delete_usrattr get_usrattr valid_usrattrconf nosubjattr
                                 subj_attr subj_callerattr subj_participants 
                                 subj_resrcattr user_bind_subject
proof
  fix sattr
  fix uattr
  show "find_usrid (subj_participants sattr) uattr \<Longrightarrow>
       user_bind_subject uattr sattr =
       subj_attr uattr (subj_participants sattr)
        (subj_resrcattr sattr)" by (auto simp: user_bind_subject_def)
next
  fix sattr
  fix uattr
  show "\<not> find_usrid (subj_participants sattr) uattr \<Longrightarrow>
       user_bind_subject uattr sattr = nosubjattr" by (auto simp: user_bind_subject_def)
qed




end