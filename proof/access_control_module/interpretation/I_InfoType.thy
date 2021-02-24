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
