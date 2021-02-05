theory I_InfoType
  imports Main "../InfoType"
begin

datatype InfoType = is_data:Data | Func

print_theorems


interpretation InfoType : InfoType Data Func is_data
proof
  show "is_data Data" by auto
next
  show "\<not> is_data Func" by auto
next
  fix x
  show "x = Data \<or> x = Func"
  proof (cases x)
    assume "x=Data"
    from this show "x = Data \<or> x = Func" by auto
  next
    assume "x=Func"
    from this show "x = Data \<or> x = Func" by auto
  qed
qed

end