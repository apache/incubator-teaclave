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

theory FMT_MSA
  imports Main ResrcAttr
begin

locale SubjAttrConf=SubjAttr nogid valid_gid noiid valid_iid trusted untrusted
                             is_trusted device normal is_normal data func is_data
                             noresrcattr resrc_attr resrcattr_presrcid resrcattr_infoid
                             resrcattr_trustlevel resrcattr_presrctype resrcattr_infotype
                             nouid valid_uid nousrattr usr_attr usrattr_id nousrattrconf
                             usrattr_conf is_usrattrconf find_usrid delete_usrattr
                             get_usrattr valid_usrattrconf nosubjattr subj_attr
                             subjattr_callerattr subjattr_participants subjattr_resrcattr+
                     subjattrconf:AttrConf nogid nosubjattr nosubjattrconf subjattr_conf is_subjattrconf 
                                           subjattr_subjid find_subjid delete_subjattr get_subjattr 
                                           subjattrconf_uniq
  for nouid::'uid
    and valid_uid::"'uid\<Rightarrow>bool"
    and nogid::"'gid"
    and valid_gid::"'gid\<Rightarrow>bool"
    and noiid::"'iid"
    and valid_iid::"'iid\<Rightarrow>bool"
    and trusted::"'trustlevel"
    and untrusted::"'trustlevel"
    and is_trusted::"'trustlevel\<Rightarrow>bool"
    and device::"'presrctype"
    and normal::"'presrctype"
    and is_normal::"'presrctype\<Rightarrow>bool"
    and data::"'infotype"
    and func::"'infotype"
    and is_data::"'infotype\<Rightarrow>bool" 
    and noresrcattr::'resrcattr
    and resrc_attr::"'gid\<Rightarrow>'iid\<Rightarrow>'trustlevel\<Rightarrow>'presrctype\<Rightarrow>'infotype\<Rightarrow>'resrcattr"
    and resrcattr_presrcid::"'resrcattr\<Rightarrow>'gid"
    and resrcattr_infoid::"'resrcattr\<Rightarrow>'iid"
    and resrcattr_trustlevel::"'resrcattr\<Rightarrow>'trustlevel"
    and resrcattr_presrctype::"'resrcattr\<Rightarrow>'presrctype"
    and resrcattr_infotype::"'resrcattr\<Rightarrow>'infotype"
    and nousrattr::'usrattr
    and usr_attr::"'uid\<Rightarrow>'usrattr"
    and usrattr_id::"'usrattr\<Rightarrow>'uid"
    and nousrattrconf::"'usrattrconf"
    and usrattr_conf::"'usrattrconf\<Rightarrow>'usrattr\<Rightarrow>'usrattrconf"
    and is_usrattrconf::"'usrattrconf\<Rightarrow>bool"
    and find_usrid::"'usrattrconf\<Rightarrow>'usrattr\<Rightarrow>bool" 
    and delete_usrattr::"'usrattrconf\<Rightarrow>'usrattr\<Rightarrow>'usrattrconf"
    and get_usrattr::"'usrattrconf\<Rightarrow>'uid\<Rightarrow>'usrattr"
    and valid_usrattrconf::"'usrattrconf\<Rightarrow>bool" 
    and nosubjattr::'subjattr
    and subj_attr::"'usrattr\<Rightarrow>'usrattrconf\<Rightarrow>'resrcattr\<Rightarrow>'subjattr"
    and subjattr_callerattr::"'subjattr\<Rightarrow>'usrattr"
    and subjattr_participants::"'subjattr\<Rightarrow>'usrattrconf"
    and subjattr_resrcattr::"'subjattr\<Rightarrow>'resrcattr" 
    and nosubjattrconf::"'subjattrconf" 
    and subjattr_conf::"'subjattrconf\<Rightarrow>'subjattr\<Rightarrow>'subjattrconf"
    and is_subjattrconf::"'subjattrconf\<Rightarrow>bool"
    and subjattr_subjid::"'subjattr\<Rightarrow>'gid"
    and find_subjid::"'subjattrconf\<Rightarrow>'subjattr\<Rightarrow>bool"
    and delete_subjattr::"'subjattrconf\<Rightarrow>'subjattr\<Rightarrow>'subjattrconf"
    and get_subjattr::"'subjattrconf\<Rightarrow>'gid\<Rightarrow>'subjattr"
    and subjattrconf_uniq::"'subjattrconf\<Rightarrow>bool" +
  fixes valid_subjattrconf::"'subjattrconf\<Rightarrow>bool"
  assumes SUBJATTRCONFHLR1:"subjattr_subjid sattr=resrcattr_presrcid(subjattr_resrcattr sattr)"
  assumes SUBJATTRCONFHLR2:"\<not>valid_subjattrconf nosubjattrconf"
  assumes SUBJATTRCONFHLR3:"conf=nosubjattrconf\<and>
                           sattr\<noteq>nosubjattr\<and>
                           resrcattr_infotype(subjattr_resrcattr sattr)=func\<Longrightarrow>
                           valid_subjattrconf(subjattr_conf conf sattr)"
  assumes SUBJATTRCONFHLR4:"conf=nosubjattrconf\<and>
                           sattr=nosubjattr\<and>
                           resrcattr_infotype(subjattr_resrcattr sattr)=func\<Longrightarrow>
                           \<not>valid_subjattrconf(subjattr_conf conf sattr)"
  assumes SUBJATTRCONFHLR5:"conf=nosubjattrconf\<and>
                           sattr\<noteq>nosubjattr\<and>
                           resrcattr_infotype(subjattr_resrcattr sattr)\<noteq>func\<Longrightarrow>
                           \<not>valid_subjattrconf(subjattr_conf conf sattr)"
  assumes SUBJATTRCONFHLR6:"conf\<noteq>nosubjattrconf\<and>
                           sattr\<noteq>nosubjattr\<and>
                           resrcattr_infotype(subjattr_resrcattr sattr)=func\<and>
                           \<not>find_subjid conf sattr\<and>
                           subjattrconf_uniq conf\<Longrightarrow>
                           valid_subjattrconf(subjattr_conf conf sattr)"
  assumes SUBJATTRCONFHLR7:"conf\<noteq>nosubjattrconf\<and>
                           sattr=nosubjattr\<Longrightarrow>
                           \<not>valid_subjattrconf(subjattr_conf conf sattr)"
  assumes SUBJATTRCONFHLR8:"conf\<noteq>nosubjattrconf\<and>
                           sattr\<noteq>nosubjattr\<and>
                           resrcattr_infotype(subjattr_resrcattr sattr)\<noteq>func\<and>
                           \<not>find_subjid conf sattr\<and>
                           subjattrconf_uniq conf\<Longrightarrow>
                           \<not>valid_subjattrconf(subjattr_conf conf sattr)"
  assumes SUBJATTRCONFHLR9:"conf\<noteq>nosubjattrconf\<and>
                           sattr\<noteq>nosubjattr\<and>
                           resrcattr_infotype(subjattr_resrcattr sattr)=func\<and>
                           find_subjid conf sattr\<and>
                           subjattrconf_uniq conf\<Longrightarrow>
                           \<not>valid_subjattrconf(subjattr_conf conf sattr)"
  assumes SUBJATTRCONFHLR10:"conf\<noteq>nosubjattrconf\<and>
                           sattr\<noteq>nosubjattr\<and>
                           resrcattr_infotype(subjattr_resrcattr sattr)=func\<and>
                           \<not>find_subjid conf sattr\<and>
                           \<not>subjattrconf_uniq conf\<Longrightarrow>
                           \<not>valid_subjattrconf(subjattr_conf conf sattr)"

print_locale! SubjAttrConf

locale ObjAttrConf=ObjAttr nogid valid_gid noiid valid_iid trusted untrusted
                           is_trusted device normal is_normal data func is_data
                           noresrcattr resrc_attr resrcattr_presrcid resrcattr_infoid
                           resrcattr_trustlevel resrcattr_presrctype resrcattr_infotype
                           nouid valid_uid nousrattr usr_attr usrattr_id nousrattrconf
                           usrattr_conf is_usrattrconf find_usrid delete_usrattr
                           get_usrattr valid_usrattrconf noobjattr obj_attr 
                           objattr_owners objattr_resrcattr +
                   objattrconf:AttrConf nogid noobjattr noobjattrconf objattr_conf is_objattrconf 
                                        objattr_objid find_objid delete_objattr get_objattr 
                                        valid_objattrconf
  for nogid::"'gid"
    and valid_gid::"'gid\<Rightarrow>bool"
    and noiid::"'iid"
    and valid_iid::"'iid\<Rightarrow>bool"
    and trusted::"'trustlevel"
    and untrusted::"'trustlevel"
    and is_trusted::"'trustlevel\<Rightarrow>bool"
    and device::"'presrctype"
    and normal::"'presrctype"
    and is_normal::"'presrctype\<Rightarrow>bool"
    and data::"'infotype"
    and func::"'infotype"
    and is_data::"'infotype\<Rightarrow>bool" 
    and noresrcattr::'resrcattr
    and resrc_attr::"'gid\<Rightarrow>'iid\<Rightarrow>'trustlevel\<Rightarrow>'presrctype\<Rightarrow>'infotype\<Rightarrow>'resrcattr"
    and resrcattr_presrcid::"'resrcattr\<Rightarrow>'gid"
    and resrcattr_infoid::"'resrcattr\<Rightarrow>'iid"
    and resrcattr_trustlevel::"'resrcattr\<Rightarrow>'trustlevel"
    and resrcattr_presrctype::"'resrcattr\<Rightarrow>'presrctype"
    and resrcattr_infotype::"'resrcattr\<Rightarrow>'infotype"
    and nouid::"'uid"
    and valid_uid::"'uid\<Rightarrow>bool" 
    and nousrattr::'usrattr
    and usr_attr::"'uid\<Rightarrow>'usrattr"
    and usrattr_id::"'usrattr\<Rightarrow>'uid"
    and nousrattrconf::"'usrattrconf"
    and usrattr_conf::"'usrattrconf\<Rightarrow>'usrattr\<Rightarrow>'usrattrconf"
    and is_usrattrconf::"'usrattrconf\<Rightarrow>bool"
    and find_usrid::"'usrattrconf\<Rightarrow>'usrattr\<Rightarrow>bool" 
    and delete_usrattr::"'usrattrconf\<Rightarrow>'usrattr\<Rightarrow>'usrattrconf"
    and get_usrattr::"'usrattrconf\<Rightarrow>'uid\<Rightarrow>'usrattr"
    and valid_usrattrconf::"'usrattrconf\<Rightarrow>bool"
    and noobjattr::'objattr
    and obj_attr::"'usrattrconf\<Rightarrow>'resrcattr\<Rightarrow>'objattr"
    and objattr_owners::"'objattr\<Rightarrow>'usrattrconf"
    and objattr_resrcattr::"'objattr\<Rightarrow>'resrcattr"
    and noobjattrconf::"'objattrconf"
    and objattr_conf::"'objattrconf\<Rightarrow>'objattr\<Rightarrow>'objattrconf"
    and is_objattrconf::"'objattrconf\<Rightarrow>bool"
    and objattr_objid::"'objattr\<Rightarrow>'gid"
    and find_objid::"'objattrconf\<Rightarrow>'objattr\<Rightarrow>bool"
    and delete_objattr::"'objattrconf\<Rightarrow>'objattr\<Rightarrow>'objattrconf"
    and get_objattr::"'objattrconf\<Rightarrow>'gid\<Rightarrow>'objattr"
    and valid_objattrconf::"'objattrconf\<Rightarrow>bool" +    
  assumes OBJATTRCONFHLR1:"objattr_objid oattr=resrcattr_presrcid(objattr_resrcattr oattr)"

print_locale! ObjAttrConf

end
