(*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 *)

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
