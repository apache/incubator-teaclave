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

theory I_ResrcAttr
  imports Main "../ResrcAttr" I_AttrConf I_TrustLevel I_ResrcType I_InfoType
begin

record ResrcAttr =
  presrc_id::ResrcId
  info_id::InfoId
  trust_level::TrustLevel
  presrc_type::PresrcType
  info_type::InfoType

definition noresrcattr::ResrcAttr where
"noresrcattr\<equiv>\<lparr>presrc_id=noresrcid, 
              info_id=noinfoid,
              trust_level=OutSgx,
              presrc_type=Normal,
              info_type=Data\<rparr>"

definition resrc_attr::"ResrcId\<Rightarrow>InfoId\<Rightarrow>TrustLevel\<Rightarrow>PresrcType\<Rightarrow>InfoType\<Rightarrow>ResrcAttr"
where
"resrc_attr pid iid tr pt it\<equiv>\<lparr>presrc_id=pid, 
                              info_id=iid,
                              trust_level=tr,
                              presrc_type=pt,
                              info_type=it\<rparr>"

interpretation ResrcAttr : ResrcAttr noresrcid valid_resrcid noinfoid valid_infoid InSgx OutSgx
                           is_insgx Device Normal is_normal Data Func is_data noresrcattr 
                           resrc_attr presrc_id info_id trust_level presrc_type info_type
proof
  show "presrc_id noresrcattr = noresrcid" by(auto simp:noresrcattr_def)
next
  show "info_id noresrcattr = noinfoid" by(auto simp:noresrcattr_def)
next
  show "trust_level noresrcattr = OutSgx" by(auto simp:noresrcattr_def)
next
  show "presrc_type noresrcattr = Normal" by(auto simp:noresrcattr_def)
next
  show "info_type noresrcattr = Data" by(auto simp:noresrcattr_def)
next
  fix pid
  fix iid
  fix tr
  fix pt
  fix it
  show "presrc_id (resrc_attr pid iid tr pt it) = pid" by(auto simp:resrc_attr_def)
next
  fix pid
  fix iid
  fix tr
  fix pt
  fix it
  show "info_id (resrc_attr pid iid tr pt it) = iid" by(auto simp:resrc_attr_def)
next
  fix pid
  fix iid
  fix tr
  fix pt
  fix it
  show "trust_level (resrc_attr pid iid tr pt it) = tr" by(auto simp:resrc_attr_def)
next
  fix pid
  fix iid
  fix tr
  fix pt
  fix it
  show "presrc_type (resrc_attr pid iid tr pt it) = pt" by(auto simp:resrc_attr_def)
next
  fix pid
  fix iid
  fix tr
  fix pt
  fix it
  show "info_type (resrc_attr pid iid tr pt it) = it" by(auto simp:resrc_attr_def)
next
  fix x
  show "\<exists>pid iid tr pt it.
       x = resrc_attr pid iid tr pt it \<or> x = noresrcattr"
  proof (rule exI)+
    show "x = resrc_attr 
                        (presrc_id x) 
                        (info_id x) 
                        (trust_level x) 
                        (presrc_type x) 
                        (info_type x) \<or> x = noresrcattr" by(auto simp: resrc_attr_def)
  qed
qed
(*
datatype SubjAttr = nosubjattr |
                    is_subjattr:subj_attr 
                              (subj_callerattr:UsrAttr) 
                              (subj_participants:UsrAttrConf) 
                              (subj_resrcattr:ResrcAttr)
                    where
                    "subj_callerattr nosubjattr=nousrattr" |
                    "subj_participants nosubjattr=nousrattrconf" |
                    "subj_resrcattr nosubjattr=noresrcattr"
*)
record SubjAttr=
  subj_callerattr::UsrAttr
  subj_participants::UsrAttrConf
  subj_resrcattr::ResrcAttr

definition nosubjattr::SubjAttr where
"nosubjattr\<equiv>\<lparr>subj_callerattr=nousrattr,
             subj_participants=nousrattrconf,
             subj_resrcattr=noresrcattr\<rparr>"

definition subj_attr::"UsrAttr\<Rightarrow>UsrAttrConf\<Rightarrow>ResrcAttr\<Rightarrow>SubjAttr" where
"subj_attr uattr uattrconf rattr\<equiv>\<lparr>subj_callerattr=uattr,
                                  subj_participants=uattrconf,
                                  subj_resrcattr=rattr\<rparr>"

interpretation SubjAttr : SubjAttr noresrcid valid_resrcid noinfoid valid_infoid InSgx OutSgx
                                   is_insgx Device Normal is_normal Data Func is_data
                                   noresrcattr resrc_attr presrc_id info_id
                                   trust_level presrc_type info_type
                                   nousrid valid_usrid nousrattr usr_attr usrattr_id nousrattrconf
                                   usrattr_conf is_usrattrconf find_usrid delete_usrattr
                                   get_usrattr valid_usrattrconf nosubjattr subj_attr
                                   subj_callerattr subj_participants subj_resrcattr
proof
  show "subj_callerattr nosubjattr = nousrattr" by (auto simp:nosubjattr_def)
next
  show "subj_participants nosubjattr = nousrattrconf" by (auto simp:nosubjattr_def)
next
  show "subj_resrcattr nosubjattr = noresrcattr" by (auto simp:nosubjattr_def)
next
  fix uattr
  fix conf
  fix attr
  show "subj_callerattr (subj_attr uattr conf attr) = uattr" by (auto simp:subj_attr_def)
next
  fix uattr
  fix conf
  fix attr
  show "subj_participants (subj_attr uattr conf attr) = conf" by (auto simp:subj_attr_def)
next
  fix uattr
  fix conf
  fix attr
  show "subj_resrcattr (subj_attr uattr conf attr) = attr" by (auto simp:subj_attr_def)
next
  fix x
  show "\<exists>uattr conf attr.
       x = subj_attr uattr conf attr \<or> x = nosubjattr"
  proof (rule exI)+
    show "x=subj_attr (subj_callerattr x) (subj_participants x) (subj_resrcattr x)\<or>
         x=nosubjattr" by (auto simp:subj_attr_def)
  qed
qed

record ObjAttr=
  obj_owners::UsrAttrConf
  obj_resrcattr::ResrcAttr

definition noobjattr::ObjAttr where
"noobjattr=\<lparr>obj_owners=nousrattrconf,
            obj_resrcattr=noresrcattr\<rparr>"

definition obj_attr::"UsrAttrConf\<Rightarrow>ResrcAttr\<Rightarrow>ObjAttr" where
"obj_attr uaconf rattr\<equiv>\<lparr>obj_owners=uaconf,
                        obj_resrcattr=rattr\<rparr>"

interpretation ObjAttr : ObjAttr noresrcid valid_resrcid noinfoid valid_infoid InSgx OutSgx
                                 is_insgx Device Normal is_normal Data Func is_data
                                 noresrcattr resrc_attr presrc_id info_id
                                 trust_level presrc_type info_type
                                 nousrid valid_usrid nousrattr usr_attr usrattr_id nousrattrconf
                                 usrattr_conf is_usrattrconf find_usrid delete_usrattr
                                 get_usrattr valid_usrattrconf noobjattr obj_attr
                                 obj_owners obj_resrcattr
proof
  show "obj_owners noobjattr = nousrattrconf" by (auto simp:noobjattr_def)
next
  show "obj_resrcattr noobjattr = noresrcattr" by (auto simp:noobjattr_def)
next
  fix conf
  fix attr
  show "obj_owners (obj_attr conf attr) = conf" by (auto simp:obj_attr_def)
next
  fix conf
  fix attr
  show "obj_resrcattr (obj_attr conf attr) = attr" by (auto simp:obj_attr_def)
next
  fix x
  show "\<exists>conf attr. x = obj_attr conf attr \<or> x = noobjattr"
  proof (rule exI)+
    show "x=obj_attr (obj_owners x) (obj_resrcattr x)\<or>
         x = noobjattr" by (auto simp:obj_attr_def)
  qed
qed
(*
datatype InfoAttr = noinfoattr |
                    is_infoattr:info_attr 
                              (info_owners:UsrAttrConf) 
                              (info_resrcattr:ResrcAttr)
                    where
                    "info_owners noinfoattr=nousrattrconf" |
                    "info_resrcattr noinfoattr=noresrcattr"
*)
record InfoAttr=
  info_owners::UsrAttrConf
  info_resrcattr::ResrcAttr

definition noinfoattr::InfoAttr where
"noinfoattr\<equiv>\<lparr>info_owners=nousrattrconf,
             info_resrcattr=noresrcattr\<rparr>"

definition info_attr::"UsrAttrConf\<Rightarrow>ResrcAttr\<Rightarrow>InfoAttr" where
"info_attr uaconf rattr\<equiv>\<lparr>info_owners=uaconf,
                         info_resrcattr=rattr\<rparr>"

interpretation InfoAttr : InfoAttr noresrcid valid_resrcid noinfoid valid_infoid InSgx OutSgx
                                   is_insgx Device Normal is_normal Data Func is_data
                                   noresrcattr resrc_attr presrc_id info_id
                                   trust_level presrc_type info_type
                                   nousrid valid_usrid nousrattr usr_attr usrattr_id nousrattrconf
                                   usrattr_conf is_usrattrconf find_usrid delete_usrattr
                                   get_usrattr valid_usrattrconf noinfoattr info_attr 
                                   info_owners info_resrcattr
proof
  show "info_owners noinfoattr = nousrattrconf" by (auto simp:noinfoattr_def)
next
  show "info_resrcattr noinfoattr = noresrcattr" by (auto simp:noinfoattr_def)
next
  fix conf
  fix attr
  show "info_owners (info_attr conf attr) = conf" by (auto simp:info_attr_def)
next
  fix conf
  fix attr
  show "info_resrcattr (info_attr conf attr) = attr" by (auto simp:info_attr_def)
next
  fix x
  show "\<exists>conf attr. x = info_attr conf attr \<or> x = noinfoattr"
  proof (rule exI)+
    show "x = info_attr (info_owners x) (info_resrcattr x) \<or> x = noinfoattr"
      by (auto simp:info_attr_def)
  qed
qed



end