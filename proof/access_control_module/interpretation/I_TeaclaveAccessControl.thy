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

theory I_TeaclaveAccessControl
  imports Main "../TeaclaveAccessControl" I_FDP_ACF I_FIA_USB I_FDP_ACC
begin

interpretation TeaclaveAccessControl:
  TeaclaveAccessControl noresrcid valid_resrcid noinfoid valid_infoid InSgx
                        OutSgx is_insgx Device Normal is_normal Data Func
                        is_data noresrcattr resrc_attr presrc_id
                        info_id trust_level presrc_type info_type nousrid valid_usrid 
                        nousrattr usr_attr usrattr_id nousrattrconf usrattr_conf is_usrattrconf
                        find_usrid delete_usrattr get_usrattr valid_usrattrconf
                        nosubjattr subj_attr subj_callerattr
                        subj_participants subj_resrcattr noobjattr obj_attr
                        obj_owners obj_resrcattr rel_subset_usr
                        task_access_function user_access_function task_access_data
                        user_access_data user_bind_subject nosubjattrconf
                        subjattr_conf is_subjattrconf subjattr_subjid find_subjid
                        delete_subjattr get_subjattr subjattrconf_uniq
                        valid_subjattrconf noobjattrconf objattr_conf is_objattrconf
                        objattr_objid find_objid delete_objattr get_objattr
                        valid_objattrconf rel_disjoint nomodelconf model_conf
                        modelconf_subj modelconf_obj valid_modelconf
proof

qed

end
