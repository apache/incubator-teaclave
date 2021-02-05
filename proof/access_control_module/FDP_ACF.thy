theory FDP_ACF
  imports Main ResrcAttr
begin


locale FdpAcf1=SubjAttr nogid valid_gid noiid valid_iid trusted untrusted
                        is_trusted device normal is_normal data func is_data
                        noresrcattr resrc_attr resrcattr_presrcid resrcattr_infoid
                        resrcattr_trustlevel resrcattr_presrctype resrcattr_infotype
                        nouid valid_uid nousrattr usr_attr usrattr_id nousrattrconf 
                        usrattr_conf is_usrattrconf find_usrid delete_usrattr get_usrattr 
                        valid_usrattrconf nosubjattr subj_attr subjattr_callerattr 
                        subjattr_participants subjattr_resrcattr +
                ObjAttr nogid valid_gid noiid valid_iid trusted untrusted
                        is_trusted device normal is_normal data func is_data
                        noresrcattr resrc_attr resrcattr_presrcid resrcattr_infoid
                        resrcattr_trustlevel resrcattr_presrctype resrcattr_infotype
                        nouid valid_uid nousrattr usr_attr usrattr_id nousrattrconf 
                        usrattr_conf is_usrattrconf find_usrid delete_usrattr get_usrattr 
                        valid_usrattrconf noobjattr obj_attr objattr_owners 
                        objattr_resrcattr +
                AttrConfRel nouid nousrattr nousrattrconf usrattr_conf is_usrattrconf usrattr_id
                            find_usrid delete_usrattr get_usrattr valid_usrattrconf rel_subset
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
    and nosubjattr::'subjattr
    and subj_attr::"'usrattr\<Rightarrow>'usrattrconf\<Rightarrow>'resrcattr\<Rightarrow>'subjattr"
    and subjattr_callerattr::"'subjattr\<Rightarrow>'usrattr"
    and subjattr_participants::"'subjattr\<Rightarrow>'usrattrconf"
    and subjattr_resrcattr::"'subjattr\<Rightarrow>'resrcattr"
    and noobjattr::'objattr
    and obj_attr::"'usrattrconf\<Rightarrow>'resrcattr\<Rightarrow>'objattr"
    and objattr_owners::"'objattr\<Rightarrow>'usrattrconf"
    and objattr_resrcattr::"'objattr\<Rightarrow>'resrcattr" 
    and rel_subset::"'usrattrconf\<Rightarrow>'usrattrconf\<Rightarrow>bool" +
  fixes task_access_function::"'subjattr\<Rightarrow>'objattr\<Rightarrow>bool"
    and user_access_function::"'usrattr\<Rightarrow>'objattr\<Rightarrow>bool"
    and task_access_data::"'subjattr\<Rightarrow>'objattr\<Rightarrow>bool"
    and user_access_data::"'usrattr\<Rightarrow>'objattr\<Rightarrow>bool"
  assumes FDPACF1HLR1:"rel_subset(subjattr_participants sattr) (objattr_owners oattr)\<and>
                      resrcattr_infotype(objattr_resrcattr oattr)=func\<Longrightarrow>
                      task_access_function sattr oattr"
  assumes FDPACF1HLR2:"\<not>rel_subset(subjattr_participants sattr) (objattr_owners oattr)\<and>
                      resrcattr_infotype(objattr_resrcattr oattr)=func\<Longrightarrow>
                      \<not>task_access_function sattr oattr"
  assumes FDPACF1HLR3:"rel_subset(subjattr_participants sattr) (objattr_owners oattr)\<and>
                      \<not>resrcattr_infotype(objattr_resrcattr oattr)\<noteq>func\<Longrightarrow>
                      task_access_function sattr oattr"  
  assumes FDPACF1HLR4:"find_usrid(objattr_owners oattr) uattr \<and>
                      (resrcattr_infotype(objattr_resrcattr oattr)=func)\<Longrightarrow>
                      user_access_function uattr oattr"
  assumes FDPACF1HLR5:"(\<not>find_usrid(objattr_owners oattr) uattr) \<and>
                      (resrcattr_infotype(objattr_resrcattr oattr)=func)\<Longrightarrow>
                      \<not>user_access_function uattr oattr"
  assumes FDPACF1HLR6:"(find_usrid(objattr_owners oattr) uattr) \<and>
                      (resrcattr_infotype(objattr_resrcattr oattr)\<noteq>func)\<Longrightarrow>
                      \<not>user_access_function uattr oattr"
  assumes FDPACF1HLR7:"rel_subset(subjattr_participants sattr) (objattr_owners oattr)\<and>
                      resrcattr_infotype(objattr_resrcattr oattr)=data\<Longrightarrow>
                      task_access_data sattr oattr"
  assumes FDPACF1HLR8:"\<not>rel_subset(subjattr_participants sattr) (objattr_owners oattr)\<and>
                      resrcattr_infotype(objattr_resrcattr oattr)=data\<Longrightarrow>
                      \<not>task_access_data sattr oattr"
  assumes FDPACF1HLR9:"rel_subset(subjattr_participants sattr) (objattr_owners oattr)\<and>
                      resrcattr_infotype(objattr_resrcattr oattr)\<noteq>data\<Longrightarrow>
                      \<not>task_access_data sattr oattr"
  assumes FDPACF1HLR10:"find_usrid(objattr_owners oattr) uattr \<and>
                      (resrcattr_infotype(objattr_resrcattr oattr)=data)\<Longrightarrow>
                      user_access_data uattr oattr"
  assumes FDPACF1HLR11:"(\<not>find_usrid(objattr_owners oattr) uattr) \<and>
                      (resrcattr_infotype(objattr_resrcattr oattr)=data)\<Longrightarrow>
                      \<not>user_access_data uattr oattr"
  assumes FDPACF1HLR12:"find_usrid(objattr_owners oattr) uattr \<and>
                      (resrcattr_infotype(objattr_resrcattr oattr)\<noteq>data)\<Longrightarrow>
                      \<not>user_access_data uattr oattr"
  assumes FDPACF1HLR13:"\<not>((find_usrid(objattr_owners oattr) uattr) \<and>
                      (resrcattr_infotype(objattr_resrcattr oattr)=data))\<Longrightarrow>
                      \<not>user_access_data uattr oattr"
  assumes FDPACF1HLR14:"\<not>(rel_subset(subjattr_participants sattr) (objattr_owners oattr)\<and>
                       resrcattr_infotype(objattr_resrcattr oattr)=data)\<Longrightarrow>
                       \<not>task_access_data sattr oattr"

begin







end



print_locale! FdpAcf1


end