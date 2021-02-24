(*
# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.
*)

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
