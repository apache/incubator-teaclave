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

theory ResrcAttr
  imports Main UsrAttr TrustLevel ResrcType InfoType
begin


locale ResrcAttr=gid:SysId nogid valid_gid +
                 iid:SysId noiid valid_iid +
                 TrustLevel trusted untrusted is_trusted +
                 PresrcType device normal is_normal +
                 InfoType data func is_data
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
    and is_data::"'infotype\<Rightarrow>bool" +
  fixes noresrcattr::'resrcattr
    and resrc_attr::"'gid\<Rightarrow>'iid\<Rightarrow>'trustlevel\<Rightarrow>'presrctype\<Rightarrow>'infotype\<Rightarrow>'resrcattr"
    and resrcattr_presrcid::"'resrcattr\<Rightarrow>'gid"
    and resrcattr_infoid::"'resrcattr\<Rightarrow>'iid"
    and resrcattr_trustlevel::"'resrcattr\<Rightarrow>'trustlevel"
    and resrcattr_presrctype::"'resrcattr\<Rightarrow>'presrctype"
    and resrcattr_infotype::"'resrcattr\<Rightarrow>'infotype"
  assumes RESRCATTRHLR1:"resrcattr_presrcid noresrcattr=nogid"
  assumes RESRCATTRHLR2:"resrcattr_infoid noresrcattr=noiid"
  assumes RESRCATTRHLR3:"resrcattr_trustlevel noresrcattr=untrusted"
  assumes RESRCATTRHLR4:"resrcattr_presrctype noresrcattr=normal"
  assumes RESRCATTRHLR5:"resrcattr_infotype noresrcattr=data"
  assumes RESRCATTRHLR6:"resrcattr_presrcid(resrc_attr pid iid tr pt it)=pid"
  assumes RESRCATTRHLR7:"resrcattr_infoid(resrc_attr pid iid tr pt it)=iid"
  assumes RESRCATTRHLR8:"resrcattr_trustlevel(resrc_attr pid iid tr pt it)=tr"
  assumes RESRCATTRHLR9:"resrcattr_presrctype(resrc_attr pid iid tr pt it)=pt"
  assumes RESRCATTRHLR10:"resrcattr_infotype(resrc_attr pid iid tr pt it)=it"
  assumes RESRCATTRHLR11:"\<exists>pid iid tr pt it. x=resrc_attr pid iid tr pt it\<or>x=noresrcattr"

print_locale! ResrcAttr

locale SubjAttr=ResrcAttr nogid valid_gid noiid valid_iid trusted untrusted
                         is_trusted device normal is_normal data func is_data
                         noresrcattr resrc_attr resrcattr_presrcid resrcattr_infoid
                         resrcattr_trustlevel resrcattr_presrctype
                         resrcattr_infotype +
                cattr:UsrAttr nouid valid_uid nousrattr usr_attr usrattr_id +
                UsrAttrConf nouid valid_uid nousrattr usr_attr usrattr_id
                            nousrattrconf usrattr_conf is_usrattrconf find_usrid
                            delete_usrattr get_usrattr valid_usrattrconf

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
    and valid_usrattrconf::"'usrattrconf\<Rightarrow>bool" +
  fixes nosubjattr::'subjattr
    and subj_attr::"'usrattr\<Rightarrow>'usrattrconf\<Rightarrow>'resrcattr\<Rightarrow>'subjattr"
    and subjattr_callerattr::"'subjattr\<Rightarrow>'usrattr"
    and subjattr_participants::"'subjattr\<Rightarrow>'usrattrconf"
    and subjattr_resrcattr::"'subjattr\<Rightarrow>'resrcattr"
  assumes SUBJATTRHLR1:"subjattr_callerattr nosubjattr=nousrattr"
  assumes SUBJATTRHLR2:"subjattr_participants nosubjattr=nousrattrconf"
  assumes SUBJATTRHLR3:"subjattr_resrcattr nosubjattr=noresrcattr"
  assumes SUBJATTRHLR4:"subjattr_callerattr(subj_attr uattr conf attr)=uattr"
  assumes SUBJATTRHLR5:"subjattr_participants(subj_attr uattr conf attr)=conf"
  assumes SUBJATTRHLR6:"subjattr_resrcattr(subj_attr uattr conf attr)=attr"
  assumes SUBJATTRHLR7:"\<exists>uattr conf attr. x=subj_attr uattr conf attr\<or>x=nosubjattr"
 

print_locale! SubjAttr

locale ObjAttr=ResrcAttr nogid valid_gid noiid valid_iid trusted untrusted
                         is_trusted device normal is_normal data func is_data
                         noresrcattr resrc_attr resrcattr_presrcid resrcattr_infoid
                         resrcattr_trustlevel resrcattr_presrctype
                         resrcattr_infotype +
               UsrAttrConf nouid valid_uid nousrattr usr_attr usrattr_id
                           nousrattrconf usrattr_conf is_usrattrconf find_usrid
                           delete_usrattr get_usrattr valid_usrattrconf
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
    and valid_usrattrconf::"'usrattrconf\<Rightarrow>bool" +
  fixes noobjattr::'objattr
    and obj_attr::"'usrattrconf\<Rightarrow>'resrcattr\<Rightarrow>'objattr"
    and objattr_owners::"'objattr\<Rightarrow>'usrattrconf"
    and objattr_resrcattr::"'objattr\<Rightarrow>'resrcattr"
  assumes OBJATTRHLR1:"objattr_owners noobjattr=nousrattrconf"
  assumes OBJATTRHLR2:"objattr_resrcattr noobjattr=noresrcattr"
  assumes OBJATTRHLR3:"objattr_owners(obj_attr conf attr)=conf"
  assumes OBJATTRHLR4:"objattr_resrcattr(obj_attr conf attr)=attr"
  assumes OBJATTRHLR5:"\<exists>conf attr. x=obj_attr conf attr\<or>x=noobjattr"

print_locale! ObjAttr

locale InfoAttr=ResrcAttr nogid valid_gid noiid valid_iid trusted untrusted
                         is_trusted device normal is_normal data func is_data
                         noresrcattr resrc_attr resrcattr_presrcid resrcattr_infoid
                         resrcattr_trustlevel resrcattr_presrctype
                         resrcattr_infotype +
                UsrAttrConf nouid valid_uid nousrattr usr_attr usrattr_id
                            nousrattrconf usrattr_conf is_usrattrconf find_usrid
                            delete_usrattr get_usrattr valid_usrattrconf
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
    and valid_usrattrconf::"'usrattrconf\<Rightarrow>bool" +
  fixes noinfoattr::'infoattr
    and info_attr::"'usrattrconf\<Rightarrow>'resrcattr\<Rightarrow>'infoattr"
    and infoattr_owners::"'infoattr\<Rightarrow>'usrattrconf"
    and infoattr_resrcattr::"'infoattr\<Rightarrow>'resrcattr"
  assumes INFOATTRHLR1:"infoattr_owners noinfoattr=nousrattrconf"
  assumes INFOATTRHLR2:"infoattr_resrcattr noinfoattr=noresrcattr"
  assumes INFOATTRHLR3:"infoattr_owners(info_attr conf attr)=conf"
  assumes INFOATTRHLR4:"infoattr_resrcattr(info_attr conf attr)=attr"
  assumes INFOATTRHLR5:"\<exists>conf attr. x=info_attr conf attr\<or>x=noinfoattr"

print_locale! InfoAttr


  

end