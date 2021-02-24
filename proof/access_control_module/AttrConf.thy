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

theory AttrConf
  imports Main
begin

locale AttrConf=
  fixes noelem::'element
    and noattr::'attr
    and noattrconf::"'attrconf"
    and attr_conf::"'attrconf\<Rightarrow>'attr\<Rightarrow>'attrconf"
    and is_attrconf::"'attrconf\<Rightarrow>bool"
    and attr_elem::"'attr\<Rightarrow>'element"
    and find_elem::"'attrconf\<Rightarrow>'attr\<Rightarrow>bool"
    and delete_attr::"'attrconf\<Rightarrow>'attr\<Rightarrow>'attrconf"
    and get_attr::"'attrconf\<Rightarrow>'element\<Rightarrow>'attr"
    and valid_attrconf::"'attrconf\<Rightarrow>bool"
  assumes ATTRCONFHLR1:"\<not>is_attrconf noattrconf"
  assumes ATTRCONFHLR2:"is_attrconf(attr_conf conf attr)"
  assumes ATTRCONFHLR3:"x=noattrconf\<or>(\<exists>conf attr. x=attr_conf conf attr)"
  assumes ATTRCONFHLR4:"attr_elem noattr=noelem"
  assumes ATTRCONFHLR5:"\<not>find_elem noattrconf attr"
  assumes ATTRCONFHLR6:"conf=noattrconf\<and>
                        attr_elem attr\<noteq>noelem\<and>
                        attr_elem attrx=attr_elem attr\<Longrightarrow>
                        find_elem(attr_conf conf attrx) attr"
  assumes ATTRCONFHLR7:"conf\<noteq>noattrconf\<and>
                        attr_elem attr\<noteq>noelem\<and>
                        attr_elem attrx=attr_elem attr\<Longrightarrow>
                        find_elem(attr_conf conf attrx) attr"
  assumes ATTRCONFHLR8:"find_elem conf attr\<and>
                        attr_elem attr=attr_elem attrx\<Longrightarrow>
                        find_elem conf attrx"
  assumes ATTRCONFHLR9:"conf\<noteq>noattrconf\<and>
                        find_elem conf attr\<Longrightarrow>
                        find_elem(attr_conf conf attrx) attr"
  assumes ATTRCONFHLR10:"\<not>((conf=noattrconf\<and>
                        attr_elem attr\<noteq>noelem\<and>
                        attr_elem attrx=attr_elem attr)\<or>
                        (conf\<noteq>noattrconf\<and>
                        attr_elem attr\<noteq>noelem\<and>
                        attr_elem attrx=attr_elem attr)\<or>
                        (conf\<noteq>noattrconf\<and>
                        find_elem conf attr))\<Longrightarrow>
                        \<not>find_elem(attr_conf conf attrx) attr"
  assumes ATTRCONFHLR11:"delete_attr noattrconf attr=noattrconf"
  assumes ATTRCONFHLR12:"attr_elem attrx=attr_elem attr\<and>
                        attr_elem attr\<noteq>noelem\<Longrightarrow>
                        delete_attr(attr_conf conf attrx) attr=conf"
  assumes ATTRCONFHLR13:"attr_elem attr=noelem\<Longrightarrow>
                        delete_attr(attr_conf conf attrx) attr=attr_conf conf attrx"
  assumes ATTRCONFHLR14:"attr_elem attrx\<noteq>attr_elem attr\<and>
                        attr_elem attr\<noteq>noelem\<Longrightarrow>
                        delete_attr(attr_conf conf attrx) attr=
                        attr_conf(delete_attr conf attr) attrx"
  assumes ATTRCONFHLR15:"get_attr noattrconf elem=noattr"
  assumes ATTRCONFHLR16:"elem\<noteq>noelem\<and>
                        attr_elem attr=elem\<Longrightarrow>
                        get_attr(attr_conf conf attr) elem=attr"
  assumes ATTRCONFHLR17:"elem=noelem\<Longrightarrow>
                        get_attr(attr_conf conf attr) elem=noattr"
  assumes ATTRCONFHLR18:"elem\<noteq>noelem\<and>
                        attr_elem attr\<noteq>elem\<Longrightarrow>
                        get_attr(attr_conf conf attr) elem=get_attr conf elem"
  assumes ATTRCONFHLR19:"\<not>valid_attrconf noattrconf"
  assumes ATTRCONFHLR20:"conf=noattrconf\<and>
                        \<not>find_elem conf attr\<Longrightarrow>
                        valid_attrconf(attr_conf conf attr)"
  assumes ATTRCONFHLR21:"conf\<noteq>noattrconf\<and>
                        \<not>find_elem conf attr\<and>
                        valid_attrconf conf\<Longrightarrow>
                        valid_attrconf(attr_conf conf attr)"
  assumes ATTRCONFHLR22:"\<not>((conf=noattrconf\<and>
                        \<not>find_elem conf attr)\<or>
                        (conf\<noteq>noattrconf\<and>
                        \<not>find_elem conf attr\<and>
                        valid_attrconf conf))\<Longrightarrow>
                        \<not>valid_attrconf(attr_conf conf attr)"
begin

lemma ATTRCONFHLR23:"noattrconf\<noteq>attr_conf conf attr"
proof
  fix conf attr
  assume 0:"noattrconf=attr_conf conf attr"
  from ATTRCONFHLR1 have "\<not>is_attrconf(attr_conf conf attr)" by(auto simp: 0)
  from this show "False" by (auto simp: ATTRCONFHLR2)
qed

lemma ATTRCONFHLR24:"x\<noteq>noattrconf\<Longrightarrow>\<exists>conf attr. x=attr_conf conf attr"
proof -
  fix x
  assume 0:"x\<noteq>noattrconf"
  from ATTRCONFHLR3 0 show "\<exists>sconf oconf. x = attr_conf sconf oconf" by blast 
qed

lemma ATTRCONFHLR25:"conf=noattrconf\<and>
                    attr_elem attrx\<noteq>attr_elem attr\<Longrightarrow>\<not>find_elem(attr_conf conf attrx) attr"
proof -
  assume "conf = noattrconf \<and> attr_elem attrx \<noteq> attr_elem attr"
  from this have "\<not>((conf=noattrconf\<and>
                  attr_elem attr\<noteq>noelem\<and>
                  attr_elem attrx=attr_elem attr)\<or>
                  (conf\<noteq>noattrconf\<and>
                  attr_elem attr\<noteq>noelem\<and>
                  attr_elem attrx=attr_elem attr)\<or>
                  (conf\<noteq>noattrconf\<and>
                  find_elem conf attr))" by blast
  from this show "\<not>find_elem(attr_conf conf attrx) attr" by (rule ATTRCONFHLR10)
qed

end

print_locale! AttrConf

locale AttrConfRel=AttrConf noelem noattr noattrconf attr_conf is_attrconf attr_elem
                            find_elem delete_attr get_attr valid_attrconf
  for noelem::'element 
    and noattr::'attr
    and noattrconf::"'attrconf"
    and attr_conf::"'attrconf\<Rightarrow>'attr\<Rightarrow>'attrconf"
    and is_attrconf::"'attrconf\<Rightarrow>bool"
    and attr_elem::"'attr\<Rightarrow>'element"
    and find_elem::"'attrconf\<Rightarrow>'attr\<Rightarrow>bool"
    and delete_attr::"'attrconf\<Rightarrow>'attr\<Rightarrow>'attrconf"
    and get_attr::"'attrconf\<Rightarrow>'element\<Rightarrow>'attr"
    and valid_attrconf::"'attrconf\<Rightarrow>bool" +
  fixes rel_subset::"'attrconf\<Rightarrow>'attrconf\<Rightarrow>bool"
  assumes ATTRCONFRELHLR1:"\<not>rel_subset confx noattrconf"
  assumes ATTRCONFRELHLR2:"conf=noattrconf\<and>
                          find_elem confx attr\<Longrightarrow>
                          rel_subset confx (attr_conf conf attr)"
  assumes ATTRCONFRELHLR3:"conf\<noteq>noattrconf\<and>
                          find_elem confx attr\<and>
                          rel_subset confx conf\<Longrightarrow>
                          rel_subset confx (attr_conf conf attr)"
  assumes ATTRCONFRELHLR4:"\<not>((conf\<noteq>noattrconf\<and>
                          find_elem confx attr\<and>
                          rel_subset confx conf)\<or>
                          (conf=noattrconf\<and>
                          find_elem confx attr))\<Longrightarrow>
                          \<not>rel_subset confx (attr_conf conf attr)"
  assumes ATTRCONFRELHLR5:"rel_subset confx conf\<and>find_elem conf attr\<Longrightarrow>find_elem confx attr"

print_locale! AttrConfRel

locale AttrConfDisj = arg1:AttrConf noelem noattr_arg1 noattrconf_arg1 attr_conf_arg1 is_attrconf_arg1 
                                    attr_elem_arg1 find_elem_arg1 delete_attr_arg1 get_attr_arg1
                                    valid_attrconf_arg1 +
                      arg2:AttrConf noelem noattr_arg2 noattrconf_arg2 attr_conf_arg2 is_attrconf_arg2
                                    attr_elem_arg2 find_elem_arg2 delete_attr_arg2 get_attr_arg2 
                                    valid_attrconf_arg2
  for noelem::'element 
    and noattr_arg1::'attr1
    and noattrconf_arg1::"'attrconf1"
    and attr_conf_arg1::"'attrconf1\<Rightarrow>'attr1\<Rightarrow>'attrconf1"
    and is_attrconf_arg1::"'attrconf1\<Rightarrow>bool"
    and attr_elem_arg1::"'attr1\<Rightarrow>'element"
    and find_elem_arg1::"'attrconf1\<Rightarrow>'attr1\<Rightarrow>bool"
    and delete_attr_arg1::"'attrconf1\<Rightarrow>'attr1\<Rightarrow>'attrconf1"
    and get_attr_arg1::"'attrconf1\<Rightarrow>'element\<Rightarrow>'attr1"
    and valid_attrconf_arg1::"'attrconf1\<Rightarrow>bool" 
    and noattr_arg2::'attr2
    and noattrconf_arg2::"'attrconf2"
    and attr_conf_arg2::"'attrconf2\<Rightarrow>'attr2\<Rightarrow>'attrconf2"
    and is_attrconf_arg2::"'attrconf2\<Rightarrow>bool"
    and attr_elem_arg2::"'attr2\<Rightarrow>'element"
    and find_elem_arg2::"'attrconf2\<Rightarrow>'attr2\<Rightarrow>bool"
    and delete_attr_arg2::"'attrconf2\<Rightarrow>'attr2\<Rightarrow>'attrconf2"
    and get_attr_arg2::"'attrconf2\<Rightarrow>'element\<Rightarrow>'attr2"
    and valid_attrconf_arg2::"'attrconf2\<Rightarrow>bool"+
  fixes rel_disjoint::"'attrconf1\<Rightarrow>'attrconf2\<Rightarrow>bool"
  assumes ATTRCONFDISJHLR1:"rel_disjoint conf1 noattrconf_arg2"
  assumes ATTRCONFDISJHLR2:"get_attr_arg1 conf1 (attr_elem_arg2 attr2)=noattr_arg1\<and>
                           rel_disjoint conf1 conf2\<Longrightarrow>
                           rel_disjoint conf1 (attr_conf_arg2 conf2 attr2)"
  assumes ATTRCONFDISJHLR3:"get_attr_arg1 conf1 (attr_elem_arg2 attr2)\<noteq>noattr_arg1\<and>
                           rel_disjoint conf1 conf2\<Longrightarrow>
                           \<not>rel_disjoint conf1 (attr_conf_arg2 conf2 attr2)"
  assumes ATTRCONFDISJHLR4:"get_attr_arg1 conf1 (attr_elem_arg2 attr2)=noattr_arg1\<and>
                           \<not>rel_disjoint conf1 conf2\<Longrightarrow>
                           \<not>rel_disjoint conf1 (attr_conf_arg2 conf2 attr2)"

print_locale! AttrConfDisj

end
