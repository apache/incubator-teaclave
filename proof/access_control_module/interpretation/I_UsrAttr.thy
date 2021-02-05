theory I_UsrAttr
  imports Main "../UsrAttr" I_SysId 
begin

record UsrAttr=
  usrattr_id::UsrId

definition usr_attr::"UsrId\<Rightarrow>UsrAttr" where
"usr_attr uid\<equiv>\<lparr>usrattr_id=uid\<rparr>"

definition nousrattr::UsrAttr where
"nousrattr\<equiv>\<lparr>usrattr_id=nousrid\<rparr>"

interpretation UsrAttr : UsrAttr nousrid valid_usrid nousrattr usr_attr usrattr_id
proof
  show "usrattr_id nousrattr = nousrid" by (auto simp:nousrattr_def)
next
  fix uid
  show " usrattr_id (usr_attr uid) = uid" by (auto simp:usr_attr_def)
next
  fix x
  show "\<exists>uid. x = usr_attr uid \<or> x = nousrattr"
  proof
    show "x = usr_attr (usrattr_id x) \<or> x = nousrattr" by (auto simp:usr_attr_def)
  qed
qed



end