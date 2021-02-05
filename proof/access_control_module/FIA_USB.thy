theory FIA_USB
  imports Main ResrcAttr
begin

locale FiaUsb1=uid:SysId nouid valid_uid +
               SubjAttr nogid valid_gid noiid valid_iid trusted untrusted
                        is_trusted device normal is_normal data func is_data
                        noresrcattr resrc_attr resrcattr_presrcid resrcattr_infoid
                        resrcattr_trustlevel resrcattr_presrctype resrcattr_infotype
                        nouid valid_uid nousrattr usr_attr usrattr_id nousrattrconf
                        usrattr_conf is_usrattrconf find_usrid delete_usrattr
                        get_usrattr valid_usrattrconf nosubjattr subj_attr
                        subjattr_callerattr subjattr_participants subjattr_resrcattr
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
    and subjattr_resrcattr::"'subjattr\<Rightarrow>'resrcattr" +
  fixes user_bind_subject::"'usrattr\<Rightarrow>'subjattr\<Rightarrow>'subjattr"
  assumes FIAUSB1HLR1:"find_usrid(subjattr_participants sattr) uattr\<Longrightarrow>
                      user_bind_subject uattr sattr=subj_attr uattr
                                                             (subjattr_participants sattr)
                                                             (subjattr_resrcattr sattr)"
  assumes FIAUSB1HLR2:"\<not>find_usrid(subjattr_participants sattr) uattr\<Longrightarrow>
                      user_bind_subject uattr sattr=nosubjattr"


print_locale! FiaUsb1



end