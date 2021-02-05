theory FDP_IFF
  imports Main ResrcAttr
begin

locale FdpIff1=fdpiff1:SubjAttr nogid valid_gid noiid valid_iid trusted untrusted
                                is_trusted device normal is_normal data func is_data
                                resrc_attr resrcattr_presrcid resrcattr_infoid
                                resrcattr_trustlevel resrcattr_presrctype resrcattr_infotype
                                nocid valid_cid nouid valid_uid nouidconf uid_conf
                                is_uidconf valid_uidconf find_uid subj_attr
                                subjattr_callerid subjattr_participants subjattr_resrcattr +
               fdpiff1:InfoAttr nogid valid_gid noiid valid_iid trusted untrusted
                                is_trusted device normal is_normal data func is_data
                                resrc_attr resrcattr_presrcid resrcattr_infoid
                                resrcattr_trustlevel resrcattr_presrctype resrcattr_infotype
                                nouid valid_uid nouidconf uid_conf is_uidconf valid_uidconf
                                find_uid info_attr infoattr_owner infoattr_resrcattr
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
    and resrc_attr::"'gid\<Rightarrow>'iid\<Rightarrow>'trustlevel\<Rightarrow>'presrctype\<Rightarrow>'infotype\<Rightarrow>'resrcattr"
    and resrcattr_presrcid::"'resrcattr\<Rightarrow>'gid"
    and resrcattr_infoid::"'resrcattr\<Rightarrow>'iid"
    and resrcattr_trustlevel::"'resrcattr\<Rightarrow>'trustlevel"
    and resrcattr_presrctype::"'resrcattr\<Rightarrow>'presrctype"
    and resrcattr_infotype::"'resrcattr\<Rightarrow>'infotype"
    and nocid::"'cid"
    and valid_cid::"'cid\<Rightarrow>bool"
    and nouid::"'uid"
    and valid_uid::"'uid\<Rightarrow>bool" 
    and nouidconf::"'uidconf"
    and uid_conf::"'uidconf\<Rightarrow>'uid\<Rightarrow>'uidconf" 
    and is_uidconf::"'uidconf\<Rightarrow>bool" 
    and valid_uidconf::"'uidconf\<Rightarrow>bool" 
    and find_uid::"'uidconf\<Rightarrow>'uid\<Rightarrow>bool"
    and subj_attr::"'cid\<Rightarrow>'uidconf\<Rightarrow>'resrcattr\<Rightarrow>'subjattr"
    and subjattr_callerid::"'subjattr\<Rightarrow>'cid"
    and subjattr_participants::"'subjattr\<Rightarrow>'uidconf"
    and subjattr_resrcattr::"'subjattr\<Rightarrow>'resrcattr"
    and info_attr::"'uidconf\<Rightarrow>'resrcattr\<Rightarrow>'infoattr"
    and infoattr_owner::"'infoattr\<Rightarrow>'uidconf"
    and infoattr_resrcattr::"'infoattr\<Rightarrow>'resrcattr"


end