theory I_TrustLevel
  imports Main "../TrustLevel"
begin

datatype TrustLevel = is_insgx:InSgx | OutSgx

print_theorems

interpretation TrustLevel : TrustLevel InSgx OutSgx is_insgx
proof
  show "is_insgx InSgx" by auto
next
  show "\<not> is_insgx OutSgx" by auto
next
  fix x
  show "x = InSgx \<or> x = OutSgx"
  proof (cases x)
    assume "x=InSgx"
    from this show "x = InSgx \<or> x = OutSgx" by auto
  next
    assume "x=OutSgx"
    from this show "x = InSgx \<or> x = OutSgx" by auto
  qed
qed




end