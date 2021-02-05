theory SysId
  imports Main
begin

locale SysId=
  fixes noid::'id
    and valid_id::"'id\<Rightarrow>bool"
  assumes SYSIDHLR1:"(x::'id)\<noteq>noid\<Longrightarrow>valid_id x"
  assumes SYSIDHLR2:"(x::'id)=noid\<Longrightarrow>\<not>valid_id x"

print_locale! SysId









end