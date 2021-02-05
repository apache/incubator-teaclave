theory InfoType
  imports Main 
begin 


locale InfoType=
  fixes data::'infotype
    and func::'infotype
    and is_data::"'infotype\<Rightarrow>bool"
  assumes INFOTYPEHLR1:"is_data data"
  assumes INFOTYPEHLR2:"\<not>is_data func"
  assumes INFOTYPEHLR3:"(x::'infotype)=data\<or>x=func"
begin

lemma INFOTYPEHLR4:"data\<noteq>func"
proof
  assume 0:"data = func"
  from INFOTYPEHLR1 have "is_data func" by(auto simp: 0)
  from this show "False" by(auto simp: INFOTYPEHLR2)
qed

end


print_locale! InfoType
                  







end