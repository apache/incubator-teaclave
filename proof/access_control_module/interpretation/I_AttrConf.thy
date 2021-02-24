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

theory I_AttrConf
  imports Main "../AttrConf" I_UsrAttr
begin

datatype UsrAttrConf = nousrattrconf |
                       is_usrattrconf:usrattr_conf UsrAttrConf UsrAttr

primrec find_usrid::"UsrAttrConf\<Rightarrow>UsrAttr\<Rightarrow>bool" where
"find_usrid nousrattrconf attr= False"
| "find_usrid (usrattr_conf conf attrx) attr=
(if usrattr_id attrx=usrattr_id attr\<and>
    usrattr_id attr\<noteq>nousrid
then
True
else find_usrid conf attr)"

primrec delete_usrattr::"UsrAttrConf\<Rightarrow>UsrAttr\<Rightarrow>UsrAttrConf" where
"delete_usrattr nousrattrconf attr=nousrattrconf"
| "delete_usrattr (usrattr_conf conf attrx) attr=
(if usrattr_id attrx=usrattr_id attr\<and>
    usrattr_id attr\<noteq>nousrid
then
conf
else if usrattr_id attrx\<noteq>usrattr_id attr\<and>
        usrattr_id attr\<noteq>nousrid
then usrattr_conf(delete_usrattr conf attr) attrx
else usrattr_conf conf attrx)"


primrec get_usrattr::"UsrAttrConf\<Rightarrow>UsrId\<Rightarrow>UsrAttr" where
"get_usrattr nousrattrconf uid=nousrattr"
| "get_usrattr (usrattr_conf conf attr) uid=
(if usrattr_id attr=uid\<and>
    uid\<noteq>nousrid
then
attr
else if usrattr_id attr\<noteq>uid\<and>
        uid\<noteq>nousrid
then
get_usrattr conf uid
else nousrattr)"

primrec valid_usrattrconf::"UsrAttrConf\<Rightarrow>bool" where
"valid_usrattrconf nousrattrconf=False"
| "valid_usrattrconf (usrattr_conf conf attr)=
(if (\<not>find_usrid conf attr)\<and>
    conf=nousrattrconf
then
True
else if conf\<noteq>nousrattrconf\<and>
        (\<not>find_usrid conf attr)\<and>
        valid_usrattrconf conf
then
True
else False)"

primrec rel_subset_usr::"UsrAttrConf\<Rightarrow>UsrAttrConf\<Rightarrow>bool" where
"rel_subset_usr conf1 nousrattrconf=False" |
"rel_subset_usr conf1 (usrattr_conf conf attr)=
(if conf=nousrattrconf\<and>
    find_usrid conf1 attr
then
True
else if conf\<noteq>nousrattrconf\<and>
        find_usrid conf1 attr
then
rel_subset_usr conf1 conf
else False)"


interpretation UsrAttrConf : UsrAttrConf nousrid valid_usrid nousrattr usr_attr usrattr_id
                                         nousrattrconf usrattr_conf is_usrattrconf find_usrid
                                         delete_usrattr get_usrattr valid_usrattrconf
proof
  show "\<not> is_usrattrconf nousrattrconf" by auto
next
  fix conf
  fix attr
  show "is_usrattrconf (usrattr_conf conf attr)" by auto
next
  fix x
  show " x = nousrattrconf \<or>
         (\<exists>conf attr. x = usrattr_conf conf attr)"
  proof (cases x)
    show "x = nousrattrconf \<Longrightarrow>
    x = nousrattrconf \<or> (\<exists>conf attr. x = usrattr_conf conf attr)" by auto
  next
    fix x21
    fix x22
    show "x = usrattr_conf x21 x22 \<Longrightarrow>
          x = nousrattrconf \<or>
          (\<exists>conf attr. x = usrattr_conf conf attr)" by auto
  qed
next
  show "usrattr_id nousrattr = nousrid" by (auto simp:nousrattr_def)
next
  fix attr
  show "\<not>find_usrid nousrattrconf attr" by auto
next
  fix conf
  fix attrx
  fix attr
  show "conf = nousrattrconf \<and>
       usrattr_id attr \<noteq> nousrid \<and>
       usrattr_id attrx = usrattr_id attr \<Longrightarrow>
       find_usrid (usrattr_conf conf attrx) attr" by (auto simp:nousrattr_def)
next
  fix conf
  fix attrx
  fix attr
  show "conf \<noteq> nousrattrconf \<and>
       usrattr_id attr \<noteq> nousrid \<and>
       usrattr_id attrx = usrattr_id attr \<Longrightarrow>
       find_usrid (usrattr_conf conf attrx) attr" by (auto simp:nousrattr_def)
next
  fix conf attr attrx
  show "find_usrid conf attr \<and> usrattr_id attr = usrattr_id attrx \<Longrightarrow>
       find_usrid conf attrx"
  proof (induct conf)
    case nousrattrconf
    then show ?case by auto
  next
    case (usrattr_conf conf x2)
    then show ?case by auto
  qed
next
  fix conf
  fix attrx
  fix attr
  show "conf \<noteq> nousrattrconf \<and> find_usrid conf attr \<Longrightarrow>
       find_usrid (usrattr_conf conf attrx) attr" by auto
next
  fix conf
  fix attrx
  fix attr
  show "\<not> (conf = nousrattrconf \<and>
           usrattr_id attr \<noteq> nousrid \<and>
           usrattr_id attrx = usrattr_id attr \<or>
           conf \<noteq> nousrattrconf \<and>
           usrattr_id attr \<noteq> nousrid \<and>
           usrattr_id attrx = usrattr_id attr \<or>
           conf \<noteq> nousrattrconf \<and> find_usrid conf attr) \<Longrightarrow>
       \<not> find_usrid (usrattr_conf conf attrx) attr" by (auto simp:nousrid_def)
next
  fix attr
  show "delete_usrattr nousrattrconf attr = nousrattrconf" by auto
next
  fix attrx
  fix attr
  fix conf
  show "usrattr_id attrx = usrattr_id attr \<and>
       usrattr_id attr \<noteq> nousrid \<Longrightarrow>
       delete_usrattr (usrattr_conf conf attrx) attr = conf" by auto
next
  fix attrx
  fix attr
  fix conf
  show "usrattr_id attr = nousrid \<Longrightarrow>
       delete_usrattr (usrattr_conf conf attrx) attr =
       usrattr_conf conf attrx" by auto
next
  fix attrx
  fix attr
  fix conf
  show "usrattr_id attrx \<noteq> usrattr_id attr \<and>
       usrattr_id attr \<noteq> nousrid \<Longrightarrow>
       delete_usrattr (usrattr_conf conf attrx) attr =
       usrattr_conf (delete_usrattr conf attr) attrx" by auto
next
  fix elem
  show "get_usrattr nousrattrconf elem = nousrattr" by auto
next
  fix attr
  fix elem
  fix conf
  show "elem \<noteq> nousrid \<and> usrattr_id attr = elem \<Longrightarrow>
       get_usrattr (usrattr_conf conf attr) elem = attr" by auto
next
  fix attr
  fix elem
  fix conf
  show "elem = nousrid \<Longrightarrow>
       get_usrattr (usrattr_conf conf attr) elem = nousrattr" by auto
next
  fix elem
  fix attr
  fix conf
  show "elem \<noteq> nousrid \<and> usrattr_id attr \<noteq> elem \<Longrightarrow>
       get_usrattr (usrattr_conf conf attr) elem =
       get_usrattr conf elem" by auto
next
  show "\<not> valid_usrattrconf nousrattrconf" by auto
next
  fix conf
  fix attr
  show "conf = nousrattrconf \<and> \<not> find_usrid conf attr \<Longrightarrow>
       valid_usrattrconf (usrattr_conf conf attr)" by auto
next
  fix conf
  fix attr
  show "conf \<noteq> nousrattrconf \<and>
       \<not> find_usrid conf attr \<and> valid_usrattrconf conf \<Longrightarrow>
       valid_usrattrconf (usrattr_conf conf attr)" by auto
next
  fix conf
  fix attr
  show "\<not> (conf = nousrattrconf \<and> \<not> find_usrid conf attr \<or>
           conf \<noteq> nousrattrconf \<and>
           \<not> find_usrid conf attr \<and> valid_usrattrconf conf) \<Longrightarrow>
       \<not> valid_usrattrconf (usrattr_conf conf attr)" by auto
qed

interpretation UsrAttrConfRel : AttrConfRel nousrid nousrattr nousrattrconf usrattr_conf is_usrattrconf 
                                usrattr_id find_usrid delete_usrattr get_usrattr valid_usrattrconf
                                rel_subset_usr
proof
  fix confx
  show "\<not> rel_subset_usr confx nousrattrconf" by auto
next
  fix conf
  fix confx
  fix attr
  show "conf = nousrattrconf \<and> find_usrid confx attr \<Longrightarrow>
       rel_subset_usr confx (usrattr_conf conf attr)" by auto
next
  fix conf
  fix confx
  fix attr
  show "conf \<noteq> nousrattrconf \<and>
       find_usrid confx attr \<and> rel_subset_usr confx conf \<Longrightarrow>
       rel_subset_usr confx (usrattr_conf conf attr)" by auto
next
  fix conf
  fix confx
  fix attr
  show "\<not> (conf \<noteq> nousrattrconf \<and>
           find_usrid confx attr \<and> rel_subset_usr confx conf \<or>
           conf = nousrattrconf \<and> find_usrid confx attr) \<Longrightarrow>
       \<not> rel_subset_usr confx (usrattr_conf conf attr)" by auto
next
  fix confx conf attr
  show "rel_subset_usr confx conf \<and> find_usrid conf attr \<Longrightarrow>
       find_usrid confx attr"
  proof (induct conf)
    case nousrattrconf
    then show ?case by auto
  next
    case (usrattr_conf conf x2)
    then show ?case
    proof (auto)
      show "(rel_subset_usr confx conf \<and> find_usrid conf attr \<Longrightarrow>
           find_usrid confx attr) \<Longrightarrow>
           if conf = nousrattrconf \<and> find_usrid confx x2 then True
           else if conf \<noteq> nousrattrconf \<and> find_usrid confx x2
           then rel_subset_usr confx conf else False \<Longrightarrow>
           if usrattr_id x2 = usrattr_id attr \<and> usrattr_id attr \<noteq> nousrid
           then True else find_usrid conf attr \<Longrightarrow>
           find_usrid confx attr"
      proof (split if_split_asm)
        show "if usrattr_id x2 = usrattr_id attr \<and> usrattr_id attr \<noteq> nousrid
             then True else find_usrid conf attr \<Longrightarrow>
             (rel_subset_usr confx conf \<and> find_usrid conf attr \<Longrightarrow>
             find_usrid confx attr) \<Longrightarrow>
             conf = nousrattrconf \<Longrightarrow>
             find_usrid confx x2 \<Longrightarrow> True \<Longrightarrow> find_usrid confx attr"
        proof (split if_split_asm)
          assume 0:"find_usrid confx x2"
          assume 1:"usrattr_id x2 = usrattr_id attr"
          from 0 1 show "find_usrid confx attr"
          proof (induct confx)
            case nousrattrconf
            then show ?case by auto
          next
            case (usrattr_conf confx x2)
            then show ?case by auto
          qed
        next
          show "(rel_subset_usr confx conf \<and> find_usrid conf attr \<Longrightarrow>
               find_usrid confx attr) \<Longrightarrow>
               conf = nousrattrconf \<Longrightarrow>
               find_usrid confx x2 \<Longrightarrow>
               True \<Longrightarrow>
               \<not> (usrattr_id x2 = usrattr_id attr \<and>
               usrattr_id attr \<noteq> nousrid) \<Longrightarrow>
               find_usrid conf attr \<Longrightarrow> find_usrid confx attr " by auto
        qed
    next
      show "if usrattr_id x2 = usrattr_id attr \<and> usrattr_id attr \<noteq> nousrid
           then True else find_usrid conf attr \<Longrightarrow>
           (rel_subset_usr confx conf \<and> find_usrid conf attr \<Longrightarrow>
           find_usrid confx attr) \<Longrightarrow>
           \<not> (conf = nousrattrconf \<and> find_usrid confx x2) \<Longrightarrow>
           if conf \<noteq> nousrattrconf \<and> find_usrid confx x2
           then rel_subset_usr confx conf else False \<Longrightarrow>
           find_usrid confx attr"
      proof (split if_split_asm)
        show "(rel_subset_usr confx conf \<and> find_usrid conf attr \<Longrightarrow>
             find_usrid confx attr) \<Longrightarrow>
             \<not> (conf = nousrattrconf \<and> find_usrid confx x2) \<Longrightarrow>
             if conf \<noteq> nousrattrconf \<and> find_usrid confx x2
             then rel_subset_usr confx conf else False \<Longrightarrow>
             usrattr_id x2 = usrattr_id attr \<Longrightarrow>
             usrattr_id attr \<noteq> nousrid \<Longrightarrow> True \<Longrightarrow> find_usrid confx attr"
          proof (split if_split_asm)
            show "usrattr_id x2 = usrattr_id attr \<Longrightarrow>
                 usrattr_id attr \<noteq> nousrid \<Longrightarrow>
                 True \<Longrightarrow>
                 (rel_subset_usr confx conf \<and> find_usrid conf attr \<Longrightarrow>
                 find_usrid confx attr) \<Longrightarrow>
                 \<not> (conf = nousrattrconf \<and> find_usrid confx x2) \<Longrightarrow>
                 \<not> (conf \<noteq> nousrattrconf \<and> find_usrid confx x2) \<Longrightarrow>
                 False \<Longrightarrow> find_usrid confx attr" by auto
          next
            assume 0:"find_usrid confx x2"
            assume 1:"usrattr_id x2 = usrattr_id attr"
            from 0 1 show "find_usrid confx attr"
            proof (induct confx)
              case nousrattrconf
              then show ?case by auto
            next
              case (usrattr_conf confx x2)
              then show ?case by auto
            qed
          qed
      next
        show "(rel_subset_usr confx conf \<and> find_usrid conf attr \<Longrightarrow>
             find_usrid confx attr) \<Longrightarrow>
             \<not> (conf = nousrattrconf \<and> find_usrid confx x2) \<Longrightarrow>
             if conf \<noteq> nousrattrconf \<and> find_usrid confx x2
             then rel_subset_usr confx conf else False \<Longrightarrow>
             \<not> (usrattr_id x2 = usrattr_id attr \<and>
             usrattr_id attr \<noteq> nousrid) \<Longrightarrow>
             find_usrid conf attr \<Longrightarrow> find_usrid confx attr"
        proof (split if_split_asm)
          show " \<not> (usrattr_id x2 = usrattr_id attr \<and>
               usrattr_id attr \<noteq> nousrid) \<Longrightarrow>
               find_usrid conf attr \<Longrightarrow>
               (rel_subset_usr confx conf \<and> find_usrid conf attr \<Longrightarrow>
               find_usrid confx attr) \<Longrightarrow>
               \<not> (conf = nousrattrconf \<and> find_usrid confx x2) \<Longrightarrow>
               conf \<noteq> nousrattrconf \<Longrightarrow>
               find_usrid confx x2 \<Longrightarrow>
               rel_subset_usr confx conf \<Longrightarrow> find_usrid confx attr" by auto
        next
          show "\<not> (usrattr_id x2 = usrattr_id attr \<and>
               usrattr_id attr \<noteq> nousrid) \<Longrightarrow>
               find_usrid conf attr \<Longrightarrow>
               (rel_subset_usr confx conf \<and> find_usrid conf attr \<Longrightarrow>
               find_usrid confx attr) \<Longrightarrow>
               \<not> (conf = nousrattrconf \<and> find_usrid confx x2) \<Longrightarrow>
               \<not> (conf \<noteq> nousrattrconf \<and> find_usrid confx x2) \<Longrightarrow>
               False \<Longrightarrow> find_usrid confx attr" by auto
          qed
        qed
      qed
    qed
  qed
qed

end
