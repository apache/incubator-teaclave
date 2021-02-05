theory ResrcType
  imports Main SysId TrustLevel
begin

locale PresrcType=
  fixes device::"'presrctype"
    and normal::"'presrctype"
    and is_normal::"'presrctype\<Rightarrow>bool"
  assumes PRESRCTYPEHLR1:"\<not>is_normal device"
  assumes PRESRCTYPEHLR2:"is_normal normal"
  assumes PRESRCTYPEHLR3:"(x::'presrctype)=device\<or>x=normal"
begin

lemma PRESRCTYPEHLR4:"device\<noteq>normal"
proof
  assume 0:"device=normal"
  from PRESRCTYPEHLR1 have "\<not>is_normal normal" by(auto simp: 0)
  from this show "False" by(auto simp: PRESRCTYPEHLR2)
qed

end

print_locale! PresrcType

locale TresrcType=tid:SysId notid valid_tid
    for notid::"'tid"
    and valid_tid::"'tid\<Rightarrow>bool"+
  fixes tresrc_type::"'tid\<Rightarrow>'tid\<Rightarrow>'tresrctype"
    and core_id::"'tresrctype\<Rightarrow>'tid"
    and period_id::"'tresrctype\<Rightarrow>'tid"
  assumes TRESRCTYPEHLR1:"core_id(tresrc_type cid pid)=cid"
  assumes TRESRCTYPEHLR2:"period_id(tresrc_type cid pid)=pid"
  assumes TRESRCTYPEHLR3:"\<exists>y::'tid. \<exists>z::'tid. x=tresrc_type y z"



print_locale! TresrcType

end