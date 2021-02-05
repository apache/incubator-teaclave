theory TrustLevel
  imports Main
begin

locale TrustLevel=
  fixes trusted::'trustlevel
    and untrusted::'trustlevel
    and is_trusted::"'trustlevel\<Rightarrow>bool"
  assumes TRUSTLEVELHLR1:"is_trusted trusted"
  assumes TRUSTLEVELHLR2:"\<not>is_trusted untrusted"
  assumes TRUSTLEVELHLR3:"(x::'trustlevel)=trusted\<or>x=untrusted"
begin

lemma TRUSTLEVELHLR4:"trusted\<noteq>untrusted"
proof
  assume 0:"trusted=untrusted"
  from TRUSTLEVELHLR1 have "is_trusted untrusted" by (auto simp: 0)
  from this show "False" by (auto simp: TRUSTLEVELHLR2)
qed

end

print_locale! TrustLevel




end