theory UsrAttr
  imports Main SysId AttrConf
begin

locale UsrAttr=SysId noid valid_id
  for noid::'id
    and valid_id::"'id\<Rightarrow>bool" +
  fixes nousrattr::'usrattr
    and usr_attr::"'id\<Rightarrow>'usrattr"
    and usrattr_id::"'usrattr\<Rightarrow>'id"
  assumes USRATTRHLR1:"usrattr_id nousrattr=noid"
  assumes USRATTRHLR2:"usrattr_id(usr_attr uid)=uid"
  assumes USRATTRHLR3:"\<exists>uid. x=usr_attr uid\<or>x=nousrattr"

print_locale! UsrAttr

locale UsrAttrConf=UsrAttr noid valid_id nousrattr usr_attr usrattr_id +
                   AttrConf noid nousrattr nousrattrconf usrattr_conf is_usrattrconf usrattr_id
                            find_usrid delete_usrattr get_usrattr valid_usrattrconf
  for noid::'id
    and valid_id::"'id\<Rightarrow>bool"
    and nousrattr::'usrattr
    and usr_attr::"'id\<Rightarrow>'usrattr"
    and usrattr_id::"'usrattr\<Rightarrow>'id"
    and nousrattrconf::"'usrattrconf"
    and usrattr_conf::"'usrattrconf\<Rightarrow>'usrattr\<Rightarrow>'usrattrconf"
    and is_usrattrconf::"'usrattrconf\<Rightarrow>bool"
    and find_usrid::"'usrattrconf\<Rightarrow>'usrattr\<Rightarrow>bool" 
    and delete_usrattr::"'usrattrconf\<Rightarrow>'usrattr\<Rightarrow>'usrattrconf"
    and get_usrattr::"'usrattrconf\<Rightarrow>'id\<Rightarrow>'usrattr"
    and valid_usrattrconf::"'usrattrconf\<Rightarrow>bool"


print_locale! UsrAttrConf


end